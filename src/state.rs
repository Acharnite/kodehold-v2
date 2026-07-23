use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleState {
    Init,
    Active,
    Review,
    Closed,
    Reopen(u32),
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecycleState::Init => write!(f, "INIT"),
            LifecycleState::Active => write!(f, "ACTIVE"),
            LifecycleState::Review => write!(f, "REVIEW"),
            LifecycleState::Closed => write!(f, "CLOSED"),
            LifecycleState::Reopen(n) => write!(f, "REOPEN({n})"),
        }
    }
}

impl LifecycleState {
    pub fn tag_name(&self) -> String {
        match self {
            LifecycleState::Init => "v2-init".into(),
            LifecycleState::Active => "v2-active".into(),
            LifecycleState::Review => "v2-review".into(),
            LifecycleState::Closed => "v2-closed".into(),
            LifecycleState::Reopen(n) => format!("v2-reopen-{n}"),
        }
    }

    pub fn from_tag(name: &str) -> Option<Self> {
        match name {
            "v2-init" => Some(LifecycleState::Init),
            "v2-active" => Some(LifecycleState::Active),
            "v2-review" => Some(LifecycleState::Review),
            "v2-closed" => Some(LifecycleState::Closed),
            s if s.starts_with("v2-reopen-") => {
                let n = s.strip_prefix("v2-reopen-")?.parse().ok()?;
                Some(LifecycleState::Reopen(n))
            }
            _ => None,
        }
    }
}

pub struct StateSnapshot {
    pub current: LifecycleState,
    pub history: Vec<(LifecycleState, String)>, // (state, tag_message)
    pub commit_count: usize,
    pub latest_tag: Option<String>,
}

pub fn derive_state(repo_path: &std::path::Path) -> anyhow::Result<StateSnapshot> {
    let repo = git2::Repository::open(repo_path)?;

    let mut tags: Vec<(LifecycleState, String)> = Vec::new();
    repo.tag_foreach(|id, _| {
        if let Ok(tag) = repo.find_tag(id) {
            let name = String::from_utf8_lossy(tag.name_bytes()).to_string();
            let msg = tag
                .message()
                .unwrap_or("")
                .trim()
                .to_string();
            if let Some(state) = LifecycleState::from_tag(&name) {
                tags.push((state, msg));
            }
        }
        true
    })?;

    // Sort by tag creation time (chronological)
    tags.sort_by_key(|(state, _)| match state {
        LifecycleState::Init => 0u32,
        LifecycleState::Active => 1,
        LifecycleState::Review => 2,
        LifecycleState::Closed => 3,
        LifecycleState::Reopen(n) => 4 + n,
    });

    let current = tags
        .last()
        .map(|(s, _)| s.clone())
        .unwrap_or(LifecycleState::Init);

    let commit_count = repo.revparse("HEAD").ok().map(|_| {
        let mut revwalk = repo.revwalk().unwrap();
        let _ = revwalk.push_head();
        revwalk.count()
    }).unwrap_or(0);

    let latest_tag = repo
        .describe(&git2::DescribeOptions::new())
        .ok()
        .and_then(|d| d.format(Some(&git2::DescribeFormatOptions::new())).ok());

    Ok(StateSnapshot {
        current,
        history: tags,
        commit_count,
        latest_tag,
    })
}

pub fn tag_project(
    repo_path: &std::path::Path,
    state: &LifecycleState,
    message: &str,
) -> anyhow::Result<()> {
    let repo = git2::Repository::open(repo_path)?;
    let obj = repo.revparse_single("HEAD")?;
    repo.tag(
        &state.tag_name(),
        &obj,
        &repo.signature()?,
        message,
        false,
    )?;
    Ok(())
}

// Kept for Phase 2 when gate transitions need tag conflict checks
#[allow(dead_code)]
pub fn check_tag_not_exists(repo_path: &std::path::Path, state: &LifecycleState) -> anyhow::Result<bool> {
    let repo = git2::Repository::open(repo_path)?;
    let ref_name = format!("refs/tags/{}", state.tag_name());
    let result = repo.find_reference(&ref_name);
    Ok(result.is_err())
}
