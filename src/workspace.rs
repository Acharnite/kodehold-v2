use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::state;

pub struct Workspace {
    pub root: PathBuf,
    pub config: Config,
}

impl Workspace {
    pub fn open(path: Option<&str>) -> anyhow::Result<Self> {
        let path = path.map(PathBuf::from).unwrap_or(std::env::current_dir().unwrap_or_default());
        let root = find_project_root(&path)?;
        let config = Config::load(&root)?;
        Ok(Self { root, config })
    }

    pub fn init(name: &str, language: Option<&str>, parent: &Path) -> anyhow::Result<Self> {
        let root = parent.join(name);

        if root.exists() {
            anyhow::bail!("directory already exists: {}", root.display());
        }

        std::fs::create_dir_all(&root)?;

        let config = Config::default_for(name, language);
        config.save(&root)?;

        // Create .gitignore
        let gitignore = match config.language.as_str() {
            "rust" => "/target\n",
            "python" => "*.pyc\n__pycache__/\n.venv/\n",
            _ => "",
        };
        std::fs::write(root.join(".gitignore"), gitignore)?;

        // git init
        let repo = git2::Repository::init(&root)?;

        // Create initial project directories
        std::fs::create_dir_all(root.join(".kodehold"))?;

        // Initial commit
        {
            let mut index = repo.index()?;
            index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            let sig = repo.signature()?;
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
        }

        // Tag v2-init
        state::tag_project(
            &root,
            &state::LifecycleState::Init,
            &format!("KodeHold v2.0 — {name} initialized"),
        )?;

        Ok(Self { root, config })
    }

    pub fn verify(&self) -> anyhow::Result<Vec<CheckResult>> {
        let mut results = Vec::new();

        let state_snapshot = state::derive_state(&self.root)?;

        // Check git state
        let repo = git2::Repository::open(&self.root)?;
        let statuses = repo.statuses(Some(git2::StatusOptions::new().include_untracked(true)))?;
        let uncommitted = statuses.iter().count();
        if uncommitted > 0 {
            results.push(CheckResult::warn(
                "uncommitted-files",
                format!("{uncommitted} uncommitted file(s)"),
            ));
        } else {
            results.push(CheckResult::pass("uncommitted-files", "Clean working tree"));
        }

        // Check tags
        let tag_name = state_snapshot.current.tag_name();
        results.push(CheckResult::pass("state", format!("State: {} (tag {})", state_snapshot.current, tag_name)));

        // Run pipeline commands
        for cmd in &self.config.pipeline.verify {
            results.push(run_cmd(cmd, &self.root));
        }

        Ok(results)
    }

    pub fn ship(&self, bump: Option<&str>) -> anyhow::Result<ShipResult> {
        let state_snapshot = state::derive_state(&self.root)?;

        // Parse current version
        let current_ver = semver::Version::parse(&self.config.version)
            .map_err(|_| anyhow::anyhow!("Invalid version in kodehold.yaml: {}", self.config.version))?;

        // Determine bump
        let new_ver = match bump.unwrap_or("patch") {
            "major" => semver::Version::new(current_ver.major + 1, 0, 0),
            "minor" => semver::Version::new(current_ver.major, current_ver.minor + 1, 0),
            "patch" => semver::Version::new(current_ver.major, current_ver.minor, current_ver.patch + 1),
            other => anyhow::bail!("Unknown bump type: {other} (use major/minor/patch)"),
        };

        // Generate changelog from commits since last tag
        let changelog_entries = generate_changelog(&self.root, state_snapshot.latest_tag.as_deref())?;

        // Build commit message
        let commit_msg = format!("chore: bump to v{new_ver}\n\n{changelog_entries}");

        // Update config
        let mut updated_config = self.config.clone();
        updated_config.version = new_ver.to_string();

        // Write updated config
        updated_config.save(&self.root)?;

        // Git commit
        let repo = git2::Repository::open(&self.root)?;
        {
            let mut index = repo.index()?;
            index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            let sig = repo.signature()?;
            let parent = repo.head()?.peel_to_commit()?;
            repo.commit(Some("HEAD"), &sig, &sig, &commit_msg, &tree, &[&parent])?;
        }

        // Tag version
        let tag_name = format!("v{new_ver}");
        let obj = repo.revparse_single("HEAD")?;
        repo.tag(
            &tag_name,
            &obj,
            &repo.signature()?,
            &format!("Version {new_ver}"),
            false,
        )?;

        Ok(ShipResult {
            old_version: current_ver.to_string(),
            new_version: new_ver.to_string(),
            tag_name,
            changelog: changelog_entries,
        })
    }
}

#[derive(Debug)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Debug)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckResult {
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self { name: name.into(), status: CheckStatus::Pass, message: message.into() }
    }
    pub fn warn(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self { name: name.into(), status: CheckStatus::Warn, message: message.into() }
    }
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self { name: name.into(), status: CheckStatus::Fail, message: message.into() }
    }
}

#[derive(Debug)]
pub struct ShipResult {
    pub old_version: String,
    pub new_version: String,
    pub tag_name: String,
    pub changelog: String,
}

fn run_cmd(cmd: &str, cwd: &std::path::Path) -> CheckResult {
    let name = cmd.split_whitespace().next().unwrap_or(cmd).to_string();
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(cwd)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            CheckResult::pass(name, "passed")
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            CheckResult::fail(name, format!("exit code {}: {}", out.status.code().unwrap_or(-1), truncate(&stderr, 200)))
        }
        Err(e) => CheckResult::fail(name, format!("failed to execute: {e}")),
    }
}

fn generate_changelog(repo_path: &Path, since_tag: Option<&str>) -> anyhow::Result<String> {
    let repo = git2::Repository::open(repo_path)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    if let Some(tag) = since_tag {
        if let Ok(obj) = repo.revparse_single(tag) {
            revwalk.hide(obj.id())?;
        }
    }

    let mut entries = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        if let Ok(commit) = repo.find_commit(oid) {
            let msg = commit.message().unwrap_or("");
            let summary = msg.lines().next().unwrap_or(msg).trim();
            if !summary.is_empty() && !summary.starts_with("chore:") {
                entries.push(format!("- {summary}"));
            }
        }
    }

    if entries.is_empty() {
        Ok("No significant changes since last release".into())
    } else {
        Ok(entries.join("\n"))
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

fn find_project_root(path: &Path) -> anyhow::Result<PathBuf> {
    let mut current = Some(path.to_path_buf());
    while let Some(dir) = current {
        if dir.join("kodehold.yaml").exists() {
            return Ok(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    anyhow::bail!("No kodehold.yaml found in {} or parent directories", path.display())
}
