pub mod pattern_db;
pub mod template;

use crate::config::Config;
use crate::pipeline::PhaseCheck;
use crate::solver::pattern_db::PatternDatabase;

pub struct SolverResult {
    pub matched: Vec<Match>,
    pub applied: Option<AppliedPattern>,
}

pub struct Match {
    pub pattern_id: String,
    pub name: String,
    pub score: f64,
    pub constraints_satisfied: bool,
}

pub struct AppliedPattern {
    pub pattern_id: String,
    pub output_files: Vec<String>,
    pub checks: Vec<PhaseCheck>,
}

pub fn solve(config: &Config) -> SolverResult {
    let db = PatternDatabase::open();
    let all = db.list(Some(&config.language));

    let matched: Vec<Match> = all
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
    }
}
