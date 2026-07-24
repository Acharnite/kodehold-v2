use std::path::{Path, PathBuf};

/// A feature request represents a unit of work to be implemented.
///
/// Feature requests can come from:
/// - Local `.khold-feature/` YAML files
/// - GitHub issues (via webhook)
/// - CLI invocation (`kodehold loop feature`)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureRequest {
    /// Short unique name for the feature (used as branch/worktree name)
    pub name: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description of what to implement
    pub description: String,
    /// Optional acceptance criteria
    #[serde(default)]
    pub acceptance: Vec<String>,
    /// Source of the feature request
    #[serde(default)]
    pub source: FeatureSource,
    /// Status in the loop lifecycle
    #[serde(default)]
    pub status: FeatureStatus,
    /// Optional labels/tags for categorization
    #[serde(default)]
    pub labels: Vec<String>,
    /// Priority (P0=critical, P1=high, P2=normal, P3=low)
    #[serde(default = "default_priority")]
    pub priority: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FeatureSource {
    /// From a local `.khold-feature/*.yaml` file
    Local,
    /// From a GitHub issue webhook event
    GitHubIssue { issue_number: u64, repo: String },
    /// From the CLI (`kodehold loop feature`)
    Cli,
    /// From a schedule / maintenance trigger
    Scheduled,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FeatureStatus {
    /// Request received, not yet started
    Pending,
    /// Worktree created, implementation in progress
    InProgress,
    /// Implementation complete, awaiting verification
    ReadyForReview,
    /// Verification passed, ready to merge
    Approved,
    /// Merged into main
    Merged,
    /// Cancelled / abandoned
    Cancelled,
}

fn default_priority() -> String {
    "P2".to_string()
}

impl Default for FeatureSource {
    fn default() -> Self {
        FeatureSource::Local
    }
}

impl Default for FeatureStatus {
    fn default() -> Self {
        FeatureStatus::Pending
    }
}

impl std::fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureStatus::Pending => write!(f, "PENDING"),
            FeatureStatus::InProgress => write!(f, "IN_PROGRESS"),
            FeatureStatus::ReadyForReview => write!(f, "READY_FOR_REVIEW"),
            FeatureStatus::Approved => write!(f, "APPROVED"),
            FeatureStatus::Merged => write!(f, "MERGED"),
            FeatureStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

/// Manages feature request files in the workspace.
pub struct FeatureStore {
    /// Directory where feature request files are stored
    pub features_dir: PathBuf,
}

impl FeatureStore {
    /// Open the feature store for a workspace.
    /// Creates the features directory if it doesn't exist.
    pub fn new(root: &Path) -> Self {
        let features_dir = root.join(".kodehold").join("features");
        Self { features_dir }
    }

    /// Ensure the features directory exists
    pub fn ensure_dir(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.features_dir)?;
        Ok(())
    }

    /// Save a feature request as a YAML file
    pub fn save(&self, feature: &FeatureRequest) -> anyhow::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.features_dir.join(format!("{}.yaml", feature.name));
        let content = serde_yaml::to_string(feature)?;
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Load a single feature request by name
    pub fn load(&self, name: &str) -> anyhow::Result<FeatureRequest> {
        let path = self.features_dir.join(format!("{name}.yaml"));
        if !path.exists() {
            anyhow::bail!("feature request '{}' not found at {}", name, path.display());
        }
        let content = std::fs::read_to_string(&path)?;
        let feature: FeatureRequest = serde_yaml::from_str(&content)?;
        Ok(feature)
    }

    /// List all feature requests, optionally filtered by status
    pub fn list(&self, status_filter: Option<FeatureStatus>) -> anyhow::Result<Vec<FeatureRequest>> {
        let mut features = Vec::new();

        if !self.features_dir.exists() {
            return Ok(features);
        }

        for entry in std::fs::read_dir(&self.features_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            match serde_yaml::from_str::<FeatureRequest>(&content) {
                Ok(feature) => {
                    if let Some(ref filter) = status_filter {
                        if feature.status == *filter {
                            features.push(feature);
                        }
                    } else {
                        features.push(feature);
                    }
                }
                Err(e) => {
                    eprintln!("  ⚠ failed to parse feature '{}': {e}", path.display());
                }
            }
        }

        // Sort by priority then name
        features.sort_by(|a, b| {
            let pa = priority_order(&a.priority);
            let pb = priority_order(&b.priority);
            pa.cmp(&pb).then(a.name.cmp(&b.name))
        });

        Ok(features)
    }

    /// Update the status of a feature request
    pub fn update_status(&self, name: &str, status: FeatureStatus) -> anyhow::Result<FeatureRequest> {
        let mut feature = self.load(name)?;
        feature.status = status;
        self.save(&feature)?;
        Ok(feature)
    }

    /// Remove a feature request file
    pub fn remove(&self, name: &str) -> anyhow::Result<()> {
        let path = self.features_dir.join(format!("{name}.yaml"));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Discover pending feature requests (those not yet started or in progress)
    pub fn discover_pending(&self) -> anyhow::Result<Vec<FeatureRequest>> {
        self.list(Some(FeatureStatus::Pending))
    }

    /// Create a feature request from a GitHub issue webhook payload
    pub fn from_github_issue(
        repo: &str,
        issue_number: u64,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> FeatureRequest {
        let name = format!("issue-{issue_number}");
        FeatureRequest {
            name,
            title: title.to_string(),
            description: body.to_string(),
            acceptance: Vec::new(),
            source: FeatureSource::GitHubIssue {
                issue_number,
                repo: repo.to_string(),
            },
            status: FeatureStatus::Pending,
            labels: labels.to_vec(),
            priority: "P2".to_string(),
        }
    }
}

fn priority_order(p: &str) -> u8 {
    match p {
        "P0" => 0,
        "P1" => 1,
        "P2" => 2,
        "P3" => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FeatureStore::new(tmp.path());

        let feature = FeatureRequest {
            name: "add-auth".to_string(),
            title: "Add authentication".to_string(),
            description: "Implement JWT-based authentication".to_string(),
            acceptance: vec!["Tokens expire after 1h".into()],
            source: FeatureSource::Cli,
            status: FeatureStatus::Pending,
            labels: vec!["enhancement".into()],
            priority: "P1".to_string(),
        };

        let path = store.save(&feature).unwrap();
        assert!(path.exists());

        let loaded = store.load("add-auth").unwrap();
        assert_eq!(loaded.name, "add-auth");
        assert_eq!(loaded.title, "Add authentication");
        assert_eq!(loaded.priority, "P1");
        assert_eq!(loaded.status, FeatureStatus::Pending);
    }

    #[test]
    fn test_list_features() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FeatureStore::new(tmp.path());

        let f1 = FeatureRequest {
            name: "feat-a".into(),
            title: "Feature A".into(),
            description: "First feature".into(),
            acceptance: vec![],
            source: FeatureSource::Cli,
            status: FeatureStatus::Pending,
            labels: vec![],
            priority: "P2".into(),
        };
        let f2 = FeatureRequest {
            name: "feat-b".into(),
            title: "Feature B".into(),
            description: "Second feature".into(),
            acceptance: vec![],
            source: FeatureSource::Cli,
            status: FeatureStatus::Merged,
            labels: vec![],
            priority: "P1".into(),
        };

        store.save(&f1).unwrap();
        store.save(&f2).unwrap();

        let all = store.list(None).unwrap();
        assert_eq!(all.len(), 2);

        let pending = store.list(Some(FeatureStatus::Pending)).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].name, "feat-a");

        let merged = store.list(Some(FeatureStatus::Merged)).unwrap();
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_from_github_issue() {
        let feature = FeatureStore::from_github_issue(
            "my-org/my-repo",
            42,
            "Add dark mode",
            "Users want dark mode support",
            &["enhancement".into(), "ui".into()],
        );

        assert_eq!(feature.name, "issue-42");
        assert_eq!(feature.title, "Add dark mode");
        assert!(matches!(feature.source, FeatureSource::GitHubIssue { .. }));
        assert_eq!(feature.labels, vec!["enhancement", "ui"]);
    }
}
