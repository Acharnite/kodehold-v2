use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub name: String,
    pub language: String,
    #[serde(default)]
    pub pipeline: PipelineConfig,
    #[serde(default)]
    pub hooks: crate::plugin::HookConfig,
    #[serde(default)]
    pub loops: crate::loops::LoopConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineConfig {
    #[serde(default = "default_phases")]
    pub phases: Vec<String>,
    #[serde(default)]
    pub verify: Vec<String>,
    #[serde(default)]
    pub optimize: Vec<String>,
}

fn default_phases() -> Vec<String> {
    vec![
        "parse".into(),
        "analyze".into(),
        "generate".into(),
        "verify".into(),
        "optimize".into(),
    ]
}

impl Config {
    pub fn default_for(name: &str, language: Option<&str>) -> Self {
        let lang = language.unwrap_or("rust").to_string();
        Self {
            version: "0.1.0".into(),
            name: name.into(),
            language: lang,
            pipeline: PipelineConfig {
                phases: default_phases(),
                verify: vec![],
                optimize: vec![],
            },
            hooks: crate::plugin::HookConfig {
                pre_generate: None,
                post_verify: None,
                on_ship: None,
            },
            loops: crate::loops::LoopConfig::default(),
        }
    }

    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.join("kodehold.yaml"))?;
        Ok(serde_yaml::from_str(&content)?)
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path.join("kodehold.yaml"), content)?;
        Ok(())
    }
}
