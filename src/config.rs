use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub name: String,
    pub language: String,
    #[serde(default)]
    pub pipeline: PipelineConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub verify: Vec<String>,
    pub optimize: Vec<String>,
}

impl Config {
    pub fn default_for(name: &str, language: Option<&str>) -> Self {
        let lang = language.unwrap_or("rust").to_string();
        let (verify, optimize) = match lang.as_str() {
            "rust" => (
                vec![],
                vec![],
            ),
            "python" => (
                vec![],
                vec![],
            ),
            _ => (vec![], vec![]),
        };
        Self {
            version: "0.1.0".into(),
            name: name.into(),
            language: lang,
            pipeline: PipelineConfig { verify, optimize },
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
