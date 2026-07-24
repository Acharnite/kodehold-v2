pub mod webhook;
pub mod watcher;

use crate::loops::feature::FeatureStore;
use crate::loops::{LoopConfig, LoopEngine};
use crate::pipeline::Pipeline;

#[derive(Debug, Clone)]
pub struct Event {
    #[allow(dead_code)]
    pub id: String,
    pub source: EventSource,
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum EventSource {
    GitHubWebhook,
    LocalWatcher,
}

impl Event {
    pub fn new(source: EventSource, event_type: &str, payload: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            event_type: event_type.to_string(),
            payload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dispatcher {
    pub workspace_path: String,
}

impl Dispatcher {
    pub fn new(path: &str) -> Self {
        Self {
            workspace_path: path.to_string(),
        }
    }

    pub fn dispatch(&self, event: &Event) -> anyhow::Result<()> {
        let action = self.route(event)?;
        match action {
            Action::FullPipeline => {
                let mut pipeline = Pipeline::new(Some(&self.workspace_path))?;
                pipeline.run(None)?;
                pipeline.report(false)?;
            }
            Action::VerifyOnly => {
                let mut pipeline = Pipeline::new(Some(&self.workspace_path))?;
                pipeline.run(Some(&["parse".into(), "analyze".into(), "verify".into()]))?;
                pipeline.report(false)?;
            }
            Action::LoopCycle => {
                let root = std::path::Path::new(&self.workspace_path);
                let config = load_loop_config(root);
                let engine = LoopEngine::new(root, config);
                match engine.run_cycle() {
                    Ok(report) => {
                        eprintln!("  ✓ loop cycle: {} features found, {} started",
                            report.features_found, report.features_started);
                        if !report.errors.is_empty() {
                            eprintln!("  ⚠ {} error(s) in cycle", report.errors.len());
                        }
                    }
                    Err(e) => eprintln!("  ✗ loop cycle error: {e}"),
                }
            }
            Action::CreateFeatureFromIssue => {
                let root = std::path::Path::new(&self.workspace_path);
                let store = FeatureStore::new(root);

                // Parse GitHub issue from payload
                if let (Some(repo_full), Some(issue_num), Some(title), Some(body)) = (
                    event.payload.get("repository").and_then(|r| r.get("full_name").and_then(|v| v.as_str())),
                    event.payload.get("issue").and_then(|i| i.get("number").and_then(|v| v.as_u64())),
                    event.payload.get("issue").and_then(|i| i.get("title").and_then(|v| v.as_str())),
                    event.payload.get("issue").and_then(|i| i.get("body").and_then(|v| v.as_str())),
                ) {
                    let labels: Vec<String> = event.payload
                        .get("issue")
                        .and_then(|i| i.get("labels"))
                        .and_then(|l| l.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|l| l.get("name").and_then(|n| n.as_str()).map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    let feature = FeatureStore::from_github_issue(repo_full, issue_num, title, body, &labels);
                    match store.save(&feature) {
                        Ok(path) => {
                            eprintln!("  ✓ created feature request from issue #{issue_num}: {title}");
                            eprintln!("    saved to: {}", path.display());
                        }
                        Err(e) => eprintln!("  ✗ failed to create feature from issue: {e}"),
                    }
                }
            }
            Action::Skip => {}
        }
        Ok(())
    }

    fn route(&self, event: &Event) -> anyhow::Result<Action> {
        match event.source {
            EventSource::GitHubWebhook => match event.event_type.as_str() {
                "push" => Ok(Action::FullPipeline),
                "pull_request" => Ok(Action::VerifyOnly),
                "issues" => {
                    let action = event.payload.get("action").and_then(|a| a.as_str()).unwrap_or("");
                    match action {
                        "opened" | "labeled" => Ok(Action::CreateFeatureFromIssue),
                        _ => Ok(Action::Skip),
                    }
                }
                "schedule" => Ok(Action::LoopCycle),
                _ => Ok(Action::Skip),
            },
            EventSource::LocalWatcher => Ok(Action::VerifyOnly),
        }
    }
}

enum Action {
    FullPipeline,
    VerifyOnly,
    LoopCycle,
    CreateFeatureFromIssue,
    Skip,
}

/// Load loop config from kodehold.yaml, falling back to defaults.
fn load_loop_config(root: &std::path::Path) -> LoopConfig {
    let yaml_path = root.join("kodehold.yaml");
    if let Ok(content) = std::fs::read_to_string(&yaml_path) {
        if let Ok(config_val) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(loops) = config_val.get("loops") {
                if let Ok(config) = serde_yaml::from_value::<LoopConfig>(loops.clone()) {
                    return config;
                }
            }
        }
    }
    LoopConfig::default()
}
