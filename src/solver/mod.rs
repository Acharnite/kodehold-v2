pub mod pattern_db;
pub mod template;

use crate::config::Config;
use crate::pipeline::PhaseCheck;
use crate::solver::pattern_db::{Pattern, PatternDatabase};

pub struct SolverResult {
    pub matched: Vec<Match>,
    #[allow(dead_code)]
    pub applied: Option<AppliedPattern>,
    pub patterns: Vec<Pattern>,
}

pub struct Match {
    pub pattern_id: String,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub score: f64,
    #[allow(dead_code)]
    pub constraints_satisfied: bool,
}

pub struct AppliedPattern {
    #[allow(dead_code)]
    pub pattern_id: String,
    #[allow(dead_code)]
    pub output_files: Vec<String>,
    #[allow(dead_code)]
    pub checks: Vec<PhaseCheck>,
}

pub fn solve(config: &Config) -> SolverResult {
    let db = PatternDatabase::open();
    let all = db.list(Some(&config.language));

    let patterns: Vec<Pattern> = all.into_iter().cloned().collect();
    let matched: Vec<Match> = patterns
        .iter()
        .map(|p| Match {
            pattern_id: p.id.clone(),
            name: p.name.clone(),
            score: 0.5,
            constraints_satisfied: true,
        })
        .collect();

    SolverResult {
        matched,
        applied: None,
        patterns,
    }
}
