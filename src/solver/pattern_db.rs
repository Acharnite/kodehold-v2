use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub signature: String,
    pub tags: Vec<String>,
    pub language: String,
    pub description: String,
    pub template: String,
    #[serde(default)]
    pub usage_count: u32,
    #[serde(default)]
    pub success_rate: f64,
}

impl Pattern {
    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.name.to_lowercase().contains(&q)
            || self.signature.to_lowercase().contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
            || self.description.to_lowercase().contains(&q)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDatabase {
    patterns: Vec<Pattern>,
    path: PathBuf,
}

impl PatternDatabase {
    pub fn open() -> Self {
        let path = default_db_path();
        let patterns = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<Vec<Pattern>>(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { patterns, path }
    }

    pub fn list(&self, language: Option<&str>) -> Vec<&Pattern> {
        match language {
            Some(lang) => self.patterns.iter().filter(|p| p.language == lang).collect(),
            None => self.patterns.iter().collect(),
        }
    }

    pub fn search(&self, query: &str) -> Vec<&Pattern> {
        self.patterns.iter().filter(|p| p.matches_query(query)).collect()
    }

    pub fn add(&mut self, pattern: Pattern) -> anyhow::Result<()> {
        // Check for duplicate ID
        if self.patterns.iter().any(|p| p.id == pattern.id) {
            anyhow::bail!("Pattern with ID '{}' already exists", pattern.id);
        }
        self.patterns.push(pattern);
        self.save()
    }

    pub fn increment_usage(&mut self, id: &str) -> anyhow::Result<()> {
        if let Some(p) = self.patterns.iter_mut().find(|p| p.id == id) {
            p.usage_count += 1;
            self.save()?;
        }
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.patterns.len()
    }

    fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.patterns)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }
}

fn default_db_path() -> PathBuf {
    if let Ok(env_path) = std::env::var("KODEHOLD_PATTERNS_PATH") {
        return PathBuf::from(env_path);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".kodehold").join("patterns.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_temp_db<F>(f: F) where F: FnOnce(PatternDatabase) {
        let tmp = std::env::temp_dir().join(format!("kodehold-patterns-{}", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        std::env::set_var("KODEHOLD_PATTERNS_PATH", &tmp);
        let db = PatternDatabase::open();
        f(db);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_empty_db() {
        with_temp_db(|db| {
            assert_eq!(db.count(), 0);
        });
    }

    #[test]
    fn test_add_and_list() {
        with_temp_db(|mut db| {
            let p = Pattern {
                id: "pat-001".into(),
                name: "test".into(),
                signature: "fn() -> ()".into(),
                tags: vec!["test".into()],
                language: "rust".into(),
                description: "Test pattern".into(),
                template: "fn main() {}".into(),
                usage_count: 0,
                success_rate: 0.0,
            };
            db.add(p).unwrap();
            assert_eq!(db.count(), 1);
            assert_eq!(db.list(Some("rust")).len(), 1);
            assert_eq!(db.list(Some("python")).len(), 0);
        });
    }

    #[test]
    fn test_search() {
        with_temp_db(|mut db| {
            db.add(Pattern {
                id: "pat-002".into(),
                name: "classifier".into(),
                signature: "fn(Vec<f64>) -> String".into(),
                tags: vec!["ml".into(), "classification".into()],
                language: "rust".into(),
                description: "A classifier pattern".into(),
                template: "".into(),
                usage_count: 0,
                success_rate: 0.0,
            }).unwrap();
            let results = db.search("classifier");
            assert_eq!(results.len(), 1);
            let results = db.search("unknown");
            assert_eq!(results.len(), 0);
        });
    }
}
