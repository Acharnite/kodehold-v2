use std::path::{Path, PathBuf};

/// Manages git worktrees for isolated feature development.
///
/// Each feature gets its own branch and linked worktree under
/// `.kodehold/worktrees/<feature-name>/`, keeping the main workspace
/// clean and allowing multiple features to be developed concurrently.
pub struct WorktreeManager {
    pub root: PathBuf,
    pub worktrees_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FeatureWorktree {
    pub name: String,
    pub branch: String,
    pub path: PathBuf,
    pub status: WorktreeStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorktreeStatus {
    Active,
    Merged,
    Abandoned,
}

impl WorktreeManager {
    pub fn new(root: &Path) -> Self {
        let worktrees_dir = root.join(".kodehold").join("worktrees");
        Self {
            root: root.to_path_buf(),
            worktrees_dir,
        }
    }

    /// Create a new feature branch and linked worktree.
    ///
    /// 1. Creates a branch `feature/<name>` from the current HEAD
    /// 2. Creates a linked worktree at `.kodehold/worktrees/<name>/`
    /// 3. Returns the worktree handle
    pub fn create_feature(&self, name: &str) -> anyhow::Result<FeatureWorktree> {
        let branch = format!("feature/{name}");
        let worktree_path = self.worktrees_dir.join(name);

        // Sanitize: ensure branch name is valid
        let safe_branch = branch.replace(' ', "-").to_lowercase();

        // Ensure worktrees dir exists
        std::fs::create_dir_all(&self.worktrees_dir)?;

        // Check if worktree already exists
        if worktree_path.exists() {
            anyhow::bail!(
                "worktree already exists at {} (branch '{safe_branch}')",
                worktree_path.display()
            );
        }

        // Create branch (from current HEAD)
        let branch_output = std::process::Command::new("git")
            .args([
                "-C",
                &self.root.to_string_lossy(),
                "branch",
                &safe_branch,
            ])
            .output()
            .map_err(|e| anyhow::anyhow!("failed to create git branch: {e}"))?;

        if !branch_output.status.success() {
            let stderr = String::from_utf8_lossy(&branch_output.stderr);
            // If branch already exists, try to use it
            if !stderr.contains("already exists") {
                anyhow::bail!("git branch failed: {stderr}");
            }
        }

        // Create linked worktree
        let wt_output = std::process::Command::new("git")
            .args([
                "-C",
                &self.root.to_string_lossy(),
                "worktree",
                "add",
                &worktree_path.to_string_lossy(),
                &safe_branch,
            ])
            .output()
            .map_err(|e| anyhow::anyhow!("failed to create git worktree: {e}"))?;

        if !wt_output.status.success() {
            let stderr = String::from_utf8_lossy(&wt_output.stderr);
            anyhow::bail!("git worktree add failed: {stderr}");
        }

        eprintln!("  ✓ created worktree: {name} -> {branch}");
        eprintln!("    path: {}", worktree_path.display());

        Ok(FeatureWorktree {
            name: name.to_string(),
            branch: safe_branch,
            path: worktree_path,
            status: WorktreeStatus::Active,
        })
    }

    /// List all active feature worktrees under `.kodehold/worktrees/`
    pub fn list_active(&self) -> anyhow::Result<Vec<FeatureWorktree>> {
        let mut worktrees = Vec::new();

        if !self.worktrees_dir.exists() {
            return Ok(worktrees);
        }

        for entry in std::fs::read_dir(&self.worktrees_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();

            // Verify this is actually a git worktree
            let git_dir = path.join(".git");
            if !git_dir.exists() {
                continue;
            }

            // Determine branch name
            let branch = format!("feature/{name}");

            // Check status via git
            let status = self.check_worktree_status(&branch);

            worktrees.push(FeatureWorktree {
                name,
                branch,
                path,
                status,
            });
        }

        // Sort by name for deterministic output
        worktrees.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(worktrees)
    }

    /// Merge a completed feature worktree back into main and clean up.
    ///
    /// Steps:
    /// 1. Fetch latest main
    /// 2. Merge feature branch into main (or rebase)
    /// 3. Remove worktree
    /// 4. Delete feature branch
    pub fn merge_feature(
        &self,
        name: &str,
        strategy: &str,
    ) -> anyhow::Result<()> {
        let branch = format!("feature/{name}");

        // Checkout main and pull latest
        self.git(&["checkout", "main"])?;
        self.git(&["pull", "--ff-only"])?;

        // Merge feature branch
        let merge_msg = format!("feat: merge {name} (kodehold loop)");
        match strategy {
            "rebase" => {
                // Rebase feature onto main, then fast-forward merge
                self.git(&["checkout", &branch])?;
                self.git(&["rebase", "main"])?;
                self.git(&["checkout", "main"])?;
                self.git(&["merge", "--ff-only", &branch])?;
            }
            "squash" => {
                // Squash all feature commits into one
                self.git(&["merge", "--squash", &branch])?;
                self.git(&["commit", "-m", &merge_msg])?;
            }
            _ => {
                // Default: regular merge with commit
                self.git(&["merge", &branch, "--no-ff", "-m", &merge_msg])?;
            }
        }

        eprintln!("  ✓ merged feature '{name}' into main ({strategy})");

        // Remove worktree
        self.remove_worktree(name)?;

        Ok(())
    }

    /// Remove a feature worktree and delete its branch.
    pub fn remove_worktree(&self, name: &str) -> anyhow::Result<()> {
        let branch = format!("feature/{name}");
        let worktree_path = self.worktrees_dir.join(name);

        // Remove worktree (force to clean up)
        let _ = self.git(&["worktree", "remove", "--force", &worktree_path.to_string_lossy()]);

        // Delete branch (might be already merged or not)
        let _ = self.git(&["branch", "-D", &branch]);

        // Remove directory if leftover
        if worktree_path.exists() {
            let _ = std::fs::remove_dir_all(&worktree_path);
        }

        eprintln!("  ✓ removed worktree: {name}");
        Ok(())
    }

    /// Check if a worktree has uncommitted changes.
    pub fn has_changes(&self, path: &Path) -> anyhow::Result<bool> {
        let output = std::process::Command::new("git")
            .args([
                "-C",
                &path.to_string_lossy(),
                "status",
                "--porcelain",
            ])
            .output()
            .map_err(|e| anyhow::anyhow!("git status failed: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.trim().is_empty())
    }

    /// Run the pipeline inside a feature worktree.
    pub fn run_pipeline_in_worktree(&self, wt: &FeatureWorktree) -> anyhow::Result<()> {
        let path_str = wt.path.to_string_lossy().to_string();

        // Use kodehold binary to run pipeline in the worktree
        let output = std::process::Command::new("kodehold")
            .args(["pipeline", "--path", &path_str])
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run pipeline in worktree: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("pipeline failed in worktree '{}': {stderr}", wt.name);
        }

        eprintln!("  ✓ pipeline passed for '{}'", wt.name);
        Ok(())
    }

    // ── helpers ──

    fn check_worktree_status(&self, branch: &str) -> WorktreeStatus {
        // Check if branch exists
        let branch_exists = self
            .git(&["branch", "--list", branch])
            .ok()
            .map(|o| !o.trim().is_empty())
            .unwrap_or(false);

        if !branch_exists {
            return WorktreeStatus::Merged;
        }

        // Check if branch is merged
        let merged = self
            .git(&["branch", "--merged", "main", "--list", branch])
            .ok()
            .map(|o| !o.trim().is_empty())
            .unwrap_or(false);

        if merged {
            WorktreeStatus::Merged
        } else {
            WorktreeStatus::Active
        }
    }

    fn git(&self, args: &[&str]) -> anyhow::Result<String> {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("-C").arg(self.root.to_string_lossy().as_ref());
        for arg in args {
            cmd.arg(arg);
        }
        let output = cmd.output().map_err(|e| anyhow::anyhow!("git error: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git {} failed: {stderr}", args.join(" "));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_repo() -> tempfile::TempDir {
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

        // Create initial commit on main
        fs::write(dir.join("README.md"), "# Test").unwrap();
        std::process::Command::new("git")
            .args(["-C", dir.to_str().unwrap(), "add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C", dir.to_str().unwrap(), "commit", "-m", "Initial commit"])
            .output()
            .unwrap();

        // Create .kodehold dir
        fs::create_dir_all(dir.join(".kodehold")).unwrap();

        tmp
    }

    #[test]
    fn test_create_feature_worktree() {
        let tmp = setup_test_repo();
        let mgr = WorktreeManager::new(tmp.path());

        let wt = mgr.create_feature("my-feature").unwrap();
        assert_eq!(wt.name, "my-feature");
        assert_eq!(wt.branch, "feature/my-feature");
        assert!(wt.path.exists());
        assert_eq!(wt.status, WorktreeStatus::Active);
    }

    #[test]
    fn test_list_active_worktrees() {
        let tmp = setup_test_repo();
        let mgr = WorktreeManager::new(tmp.path());

        mgr.create_feature("feat-a").unwrap();
        mgr.create_feature("feat-b").unwrap();

        let list = mgr.list_active().unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|w| w.name == "feat-a"));
        assert!(list.iter().any(|w| w.name == "feat-b"));
    }

    #[test]
    fn test_remove_worktree() {
        let tmp = setup_test_repo();
        let mgr = WorktreeManager::new(tmp.path());

        mgr.create_feature("to-remove").unwrap();
        mgr.remove_worktree("to-remove").unwrap();

        let list = mgr.list_active().unwrap();
        assert!(list.iter().all(|w| w.name != "to-remove"));
    }

    #[test]
    fn test_detects_uncommitted_changes() {
        let tmp = setup_test_repo();
        let mgr = WorktreeManager::new(tmp.path());
        let wt = mgr.create_feature("dirty-test").unwrap();

        // No changes initially
        assert!(!mgr.has_changes(&wt.path).unwrap());

        // Add a file
        fs::write(wt.path.join("new-file.txt"), "content").unwrap();
        assert!(mgr.has_changes(&wt.path).unwrap());
    }
}
