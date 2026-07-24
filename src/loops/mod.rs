pub mod feature;
pub mod worktree;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use feature::{FeatureRequest, FeatureStatus, FeatureStore};
use worktree::{WorktreeManager, WorktreeStatus};

/// KodeHold Loops — continuous workspace maintenance and feature integration.
///
/// The LoopEngine orchestrates the lifecycle of feature requests:
///
/// ```text
/// Pending → InProgress (worktree created, pipeline runs)
///         → ReadyForReview (verification passes)
///         → Approved (human reviews)
///         → Merged (merged back to main)
///         → Cancelled (abandoned)
/// ```
///
/// It also handles scheduled maintenance (dependency updates, cleanup).
pub struct LoopEngine {
    pub root: PathBuf,
    pub features: FeatureStore,
    pub worktrees: WorktreeManager,
    pub config: LoopConfig,
}

/// Configuration for the loop engine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopConfig {
    /// Directory for feature request files (relative to workspace root)
    #[serde(default = "default_features_dir")]
    pub features_dir: String,
    /// Directory for linked worktrees (relative to workspace root)
    #[serde(default = "default_worktrees_dir")]
    pub worktrees_dir: String,
    /// Whether to auto-run maintenance on each cycle
    #[serde(default = "default_true")]
    pub auto_maintain: bool,
    /// Interval in seconds between maintenance cycles (0 = once per loop start)
    #[serde(default = "default_interval")]
    pub maintain_interval_secs: u64,
    /// Merge strategy: "merge", "rebase", or "squash"
    #[serde(default = "default_merge_strategy")]
    pub merge_strategy: String,
    /// Whether to auto-merge features that pass verification
    #[serde(default)]
    pub auto_merge: bool,
    /// Maximum concurrent feature worktrees
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_features: usize,
    /// Labels that indicate a feature should be auto-processed
    #[serde(default)]
    pub auto_labels: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoopReport {
    pub cycle_id: String,
    pub features_found: usize,
    pub features_started: usize,
    pub features_merged: usize,
    pub maintenance_run: bool,
    pub errors: Vec<String>,
    pub worktrees_active: usize,
}

fn default_features_dir() -> String {
    ".kodehold/features".to_string()
}
fn default_worktrees_dir() -> String {
    ".kodehold/worktrees".to_string()
}
fn default_true() -> bool {
    true
}
fn default_interval() -> u64 {
    3600
}
fn default_merge_strategy() -> String {
    "merge".to_string()
}
fn default_max_concurrent() -> usize {
    3
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            features_dir: default_features_dir(),
            worktrees_dir: default_worktrees_dir(),
            auto_maintain: true,
            maintain_interval_secs: 3600,
            merge_strategy: "merge".to_string(),
            auto_merge: false,
            max_concurrent_features: 3,
            auto_labels: vec!["auto".to_string()],
        }
    }
}

impl LoopEngine {
    /// Create a new loop engine for the given workspace root.
    pub fn new(root: &Path, config: LoopConfig) -> Self {
        Self {
            root: root.to_path_buf(),
            features: FeatureStore::new(root),
            worktrees: WorktreeManager::new(root),
            config,
        }
    }

    /// Run a single loop cycle:
    /// 1. Discover pending feature requests
    /// 2. Start work on eligible features (create worktrees, run pipeline)
    /// 3. Verify in-progress features
    /// 4. Merge completed features (if auto-merge enabled)
    /// 5. Run maintenance tasks
    /// 6. Prune abandoned worktrees
    pub fn run_cycle(&self) -> anyhow::Result<LoopReport> {
        let cycle_id = uuid::Uuid::new_v4().to_string();
        let mut report = LoopReport {
            cycle_id: cycle_id.clone(),
            features_found: 0,
            features_started: 0,
            features_merged: 0,
            maintenance_run: false,
            errors: Vec::new(),
            worktrees_active: 0,
        };

        eprintln!("\n━━━ Loop Cycle {cycle_id} ━━━");

        // Step 1: Discover pending features
        let pending = match self.features.discover_pending() {
            Ok(f) => {
                report.features_found = f.len();
                eprintln!("  found {} pending feature(s)", f.len());
                f
            }
            Err(e) => {
                eprintln!("  ⚠ failed to discover features: {e}");
                report.errors.push(format!("discover features: {e}"));
                Vec::new()
            }
        };

        // Step 2: Start work on pending features (up to max_concurrent)
        let active = self.worktrees.list_active().unwrap_or_default();
        let slots_available = self.config.max_concurrent_features.saturating_sub(active.len());
        report.worktrees_active = active.len();

        let mut auto_features: Vec<FeatureRequest> = pending
            .into_iter()
            .filter(|f| self.should_auto_process(f))
            .collect();

        // Take only what fits in available slots
        let to_process: Vec<FeatureRequest> = auto_features.drain(..slots_available.min(auto_features.len())).collect();

        for feature in &to_process {
            match self.start_feature(feature) {
                Ok(_) => {
                    report.features_started += 1;
                }
                Err(e) => {
                    eprintln!("  ✗ failed to start feature '{}': {e}", feature.name);
                    report.errors.push(format!("start '{}': {e}", feature.name));
                }
            }
        }

        // Step 3: Check in-progress features — try pipeline verification
        if let Ok(active_wts) = self.worktrees.list_active() {
            for wt in &active_wts {
                // Check if this worktree has a corresponding feature request
                match self.features.load(&wt.name) {
                    Ok(feature) if feature.status == FeatureStatus::InProgress => {
                        // Run pipeline in the worktree to verify
                        match self.worktrees.run_pipeline_in_worktree(wt) {
                            Ok(_) => {
                                // Pipeline passed — mark as ReadyForReview
                                eprintln!("  ✓ feature '{}' passed verification", feature.name);
                                if let Err(e) = self.features.update_status(&feature.name, FeatureStatus::ReadyForReview) {
                                    report.errors.push(format!("update status '{}': {e}", feature.name));
                                }
                            }
                            Err(e) => {
                                eprintln!("  ⚠ feature '{}' verification failed: {e}", feature.name);
                            }
                        }
                    }
                    Ok(feature) if feature.status == FeatureStatus::ReadyForReview => {
                        if self.config.auto_merge {
                            match self.merge_feature(&feature) {
                                Ok(_) => {
                                    report.features_merged += 1;
                                }
                                Err(e) => {
                                    eprintln!("  ✗ auto-merge failed for '{}': {e}", feature.name);
                                    report.errors.push(format!("merge '{}': {e}", feature.name));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Step 4: Run maintenance if enabled
        if self.config.auto_maintain {
            match self.run_maintenance() {
                Ok(_) => {
                    report.maintenance_run = true;
                }
                Err(e) => {
                    eprintln!("  ⚠ maintenance failed: {e}");
                    report.errors.push(format!("maintenance: {e}"));
                }
            }
        }

        // Step 5: Prune abandoned worktrees
        if let Err(e) = self.prune_abandoned() {
            eprintln!("  ⚠ prune failed: {e}");
            report.errors.push(format!("prune: {e}"));
        }

        eprintln!("━━━ Cycle Complete ━━━\n");

        Ok(report)
    }

    /// Run the loop engine as a continuous daemon.
    /// Processes features and then sleeps for the given interval.
    pub fn run_daemon(
        root: &Path,
        config: LoopConfig,
        interval_secs: u64,
        single_shot: bool,
    ) -> anyhow::Result<()> {
        let engine = Self::new(root, config);

        if single_shot {
            let report = engine.run_cycle()?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(());
        }

        eprintln!("━━━ KodeHold Loop Daemon ━━━");
        eprintln!("  workspace: {}", root.display());
        eprintln!("  interval: {interval_secs}s");
        eprintln!("  Press Ctrl+C to stop\n");

        loop {
            let cycle_start = Instant::now();

            if let Err(e) = engine.run_cycle() {
                eprintln!("  ✗ cycle error: {e}");
            }

            let elapsed = cycle_start.elapsed();
            let sleep = Duration::from_secs(interval_secs).saturating_sub(elapsed);

            if sleep > Duration::from_secs(0) {
                eprintln!("  sleeping for {}s...\n", sleep.as_secs());
                std::thread::sleep(sleep);
            }
        }
    }

    // ── Feature lifecycle ──

    /// Start working on a feature: create worktree, commit initial state.
    fn start_feature(&self, feature: &FeatureRequest) -> anyhow::Result<()> {
        eprintln!("  → starting feature '{}'", feature.name);
        eprintln!("    title: {}", feature.title);

        // Create worktree
        let wt = self.worktrees.create_feature(&feature.name)?;

        // Copy feature request into worktree for reference
        let feature_path = self.features.save(feature)?;
        if let Some(parent) = wt.path.parent() {
            let kodehold_dir = parent.join(".kodehold");
            std::fs::create_dir_all(&kodehold_dir)?;
            std::fs::copy(&feature_path, kodehold_dir.join(format!("{}.yaml", feature.name)))?;
        }

        // Create initial commit in worktree
        let _ = std::process::Command::new("git")
            .args([
                "-C",
                &wt.path.to_string_lossy(),
                "add",
                ".",
            ])
            .output();
        let _ = std::process::Command::new("git")
            .args([
                "-C",
                &wt.path.to_string_lossy(),
                "commit",
                "-m",
                &format!("feat: start work on {}", feature.title),
                "--allow-empty",
            ])
            .output();

        // Update feature status to InProgress
        self.features.update_status(&feature.name, FeatureStatus::InProgress)?;

        Ok(())
    }

    /// Merge a completed feature into main.
    fn merge_feature(&self, feature: &FeatureRequest) -> anyhow::Result<()> {
        eprintln!("  → merging feature '{}'", feature.name);

        self.worktrees
            .merge_feature(&feature.name, &self.config.merge_strategy)?;

        // Update feature status
        self.features.update_status(&feature.name, FeatureStatus::Merged)?;

        // Remove feature request file (it's now part of history)
        let _ = self.features.remove(&feature.name);

        Ok(())
    }

    // ── Maintenance ──

    /// Run maintenance tasks:
    /// - Prune stale worktrees
    /// - Run verify on main
    /// - Check for git garbage
    pub(crate) fn run_maintenance(&self) -> anyhow::Result<()> {
        eprintln!("  → running maintenance...");

        // Run kodehold verify on main workspace
        let verify_output = std::process::Command::new("kodehold")
            .args(["verify", "--path", &self.root.to_string_lossy()])
            .output();

        match verify_output {
            Ok(out) if out.status.success() => {
                eprintln!("  ✓ maintenance verify passed");
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("  ⚠ maintenance verify: {stderr}");
            }
            Err(e) => {
                eprintln!("  ⚠ maintenance verify cmd failed: {e}");
            }
        }

        // Git gc (lightweight)
        let _ = std::process::Command::new("git")
            .args(["-C", &self.root.to_string_lossy(), "gc", "--auto"])
            .output();

        Ok(())
    }

    /// Prune abandoned feature worktrees (branches gone, dirs leftover).
    pub(crate) fn prune_abandoned(&self) -> anyhow::Result<()> {
        let active = self.worktrees.list_active()?;
        for wt in &active {
            if wt.status == WorktreeStatus::Merged || wt.status == WorktreeStatus::Abandoned {
                eprintln!("  → pruning worktree '{}'", wt.name);
                self.worktrees.remove_worktree(&wt.name)?;
            }
        }

        // Also run git worktree prune to clean up git internals
        let _ = std::process::Command::new("git")
            .args(["-C", &self.root.to_string_lossy(), "worktree", "prune"])
            .output();

        Ok(())
    }

    // ── Helpers ──

    /// Determine if a feature should be auto-processed based on labels and priority.
    fn should_auto_process(&self, feature: &FeatureRequest) -> bool {
        // Always process P0 features
        if feature.priority == "P0" {
            return true;
        }

        // Check if feature has any auto-process labels
        if !self.config.auto_labels.is_empty() {
            if feature.labels.iter().any(|l| self.config.auto_labels.contains(l)) {
                return true;
            }
        }

        // GitHub issues are auto-processed by default
        matches!(feature.source, feature::FeatureSource::GitHubIssue { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_workspace() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // Init git repo
        std::process::Command::new("git")
            .args(["init", dir.to_str().unwrap()])
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["-C", dir.to_str().unwrap(), "config", "user.email", "test@test.com"])
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["-C", dir.to_str().unwrap(), "config", "user.name", "Test"])
            .output()
            .unwrap();

        fs::write(dir.join("README.md"), "# Test Workspace").unwrap();
        fs::write(dir.join("kodehold.yaml"), "version: 0.1.0\nname: test\nlanguage: rust\n").unwrap();

        std::process::Command::new("git")
            .args(["-C", dir.to_str().unwrap(), "add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C", dir.to_str().unwrap(), "commit", "-m", "Initial commit"])
            .output()
            .unwrap();

        fs::create_dir_all(dir.join(".kodehold")).unwrap();

        tmp
    }

    #[test]
    fn test_run_cycle_with_no_features() {
        let tmp = setup_workspace();
        let engine = LoopEngine::new(tmp.path(), LoopConfig::default());
        let report = engine.run_cycle().unwrap();
        assert_eq!(report.features_found, 0);
        assert_eq!(report.features_started, 0);
        assert!(report.maintenance_run);
    }

    #[test]
    fn test_should_auto_process_p0() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = LoopEngine::new(tmp.path(), LoopConfig::default());

        let critical = FeatureRequest {
            name: "critical-fix".into(),
            title: "Critical fix".into(),
            description: "Fix security issue".into(),
            acceptance: vec![],
            source: feature::FeatureSource::Cli,
            status: FeatureStatus::Pending,
            labels: vec![],
            priority: "P0".into(),
        };

        assert!(engine.should_auto_process(&critical));
    }

    #[test]
    fn test_should_auto_process_github_issue() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = LoopEngine::new(tmp.path(), LoopConfig::default());

        let issue = FeatureRequest {
            name: "issue-1".into(),
            title: "Bug report".into(),
            description: "Something broke".into(),
            acceptance: vec![],
            source: feature::FeatureSource::GitHubIssue {
                issue_number: 1,
                repo: "test/repo".into(),
            },
            status: FeatureStatus::Pending,
            labels: vec!["bug".into()],
            priority: "P2".into(),
        };

        assert!(engine.should_auto_process(&issue));
    }

    #[test]
    fn test_start_feature_creates_worktree() {
        let tmp = setup_workspace();
        let engine = LoopEngine::new(tmp.path(), LoopConfig::default());

        let feature = FeatureRequest {
            name: "test-feature".into(),
            title: "Test Feature".into(),
            description: "A test feature".into(),
            acceptance: vec![],
            source: feature::FeatureSource::Cli,
            status: FeatureStatus::Pending,
            labels: vec![],
            priority: "P2".into(),
        };

        // Save feature first
        engine.features.save(&feature).unwrap();

        // Start feature
        engine.start_feature(&feature).unwrap();

        // Verify worktree exists
        let wt_path = tmp.path().join(".kodehold").join("worktrees").join("test-feature");
        assert!(wt_path.exists());

        // Verify status updated
        let loaded = engine.features.load("test-feature").unwrap();
        assert_eq!(loaded.status, FeatureStatus::InProgress);
    }
}
