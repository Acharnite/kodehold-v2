pub mod webhook;
pub mod watcher;

use std::sync::Arc;

use crate::pipeline::Pipeline;

#[derive(Debug, Clone)]
pub struct Event {
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
            Action::Skip => {}
        }
        Ok(())
    }

    fn route(&self, event: &Event) -> anyhow::Result<Action> {
        match event.source {
            EventSource::GitHubWebhook => match event.event_type.as_str() {
                "push" => Ok(Action::FullPipeline),
                "pull_request" => Ok(Action::VerifyOnly),
                _ => Ok(Action::Skip),
            },
            EventSource::LocalWatcher => Ok(Action::VerifyOnly),
        }
    }
}

enum Action {
    FullPipeline,
    VerifyOnly,
    Skip,
}
