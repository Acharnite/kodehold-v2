use crate::config::Config;
use crate::workspace::Workspace;

pub struct Pipeline {
    pub workspace: Workspace,
    pub config: Config,
    pub context: PipelineContext,
}

pub struct PipelineContext {
    pub parsed: Option<ParsedOutput>,
    pub analyzed: Option<AnalyzedOutput>,
    pub generated: Option<GeneratedOutput>,
    pub verified: Option<VerifiedOutput>,
    pub optimized: Option<OptimizedOutput>,
}

pub struct ParsedOutput {
    pub files: Vec<ParsedFile>,
}

pub struct ParsedFile {
    pub path: String,
    pub language: String,
    pub size_bytes: u64,
    pub has_kodehold_yaml: bool,
}

pub struct AnalyzedOutput {
    pub modules: Vec<String>,
    pub test_framework: Option<String>,
    pub has_cargo_toml: bool,
}

pub struct GeneratedOutput {
    pub files_written: Vec<String>,
    pub source: GenerationSource,
}

pub enum GenerationSource {
    Solver { pattern_id: String },
    #[allow(dead_code)]
    AiFallback,
    DslCompiled,
    Skipped,
}

pub struct VerifiedOutput {
    pub results: Vec<PhaseCheck>,
}

pub struct OptimizedOutput {
    pub formatted_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PhaseCheck {
    pub name: String,
    pub status: PhaseStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhaseStatus {
    Pass,
    Warn,
    Fail,
    #[allow(dead_code)]
    Skipped,
}

impl Pipeline {
    pub fn new(path: Option<&str>) -> anyhow::Result<Self> {
        let workspace = Workspace::open(path)?;
        let config = workspace.config.clone();
        Ok(Self {
            workspace,
            config,
            context: PipelineContext::default(),
        })
    }

    pub fn run(&mut self, filter: Option<&[String]>) -> anyhow::Result<()> {
        let phases: Vec<String> = if let Some(f) = filter {
            f.to_vec()
        } else {
            self.config.pipeline.phases.clone()
        };

        let hooks = crate::plugin::HookConfig::from_config(&self.config);

        for phase in &phases {
            match phase.as_str() {
                "parse" => self.run_parse()?,
                "analyze" => self.run_analyze()?,
                "generate" => {
                    // pre-generate hook
                    if let Some(ref path) = hooks.pre_generate {
                        let r = crate::plugin::run_hook(&self.workspace.root, path);
                        if r.status == crate::plugin::HookStatus::Blocked {
                            eprintln!("  ✗ hook blocked: {} ({})", r.hook, r.message);
                            anyhow::bail!("hook blocked by {hook}", hook = r.hook);
                        }
                    }
                    self.run_generate()?;
                }
                "verify" => {
                    self.run_verify()?;
                    // post-verify hook
                    if let Some(ref path) = hooks.post_verify {
                        let r = crate::plugin::run_hook(&self.workspace.root, path);
                        if r.status == crate::plugin::HookStatus::Blocked {
                            eprintln!("  ✗ hook blocked: {} ({})", r.hook, r.message);
                            anyhow::bail!("hook blocked by {hook}", hook = r.hook);
                        }
                    }
                }
                "optimize" => self.run_optimize()?,
                other => anyhow::bail!("Unknown pipeline phase: {other}"),
            }
        }
        Ok(())
    }

    pub fn run_parse(&mut self) -> anyhow::Result<()> {
        let root = &self.workspace.root;
        let mut files = Vec::new();

        // Scan project files
        for entry in walkdir(root, 3, &[".git", "target", ".venv", "node_modules"])? {
            let size = std::fs::metadata(&entry)?.len();
            let ext = entry.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang = match ext {
                "rs" => "rust",
                "py" => "python",
                "js" | "ts" => "javascript",
                "go" => "go",
                "yaml" | "yml" => "yaml",
                "json" => "json",
                "md" => "markdown",
                _ => "unknown",
            };
            files.push(ParsedFile {
                path: entry.to_string_lossy().to_string(),
                language: lang.to_string(),
                size_bytes: size,
                has_kodehold_yaml: entry.file_name().and_then(|n| n.to_str()) == Some("kodehold.yaml"),
            });
        }

        self.context.parsed = Some(ParsedOutput { files });
        Ok(())
    }

    pub fn run_analyze(&mut self) -> anyhow::Result<()> {
        let parsed = self.context.parsed.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Parse phase must run before analyze"))?;

        let mut modules = Vec::new();
        let mut test_framework = None;
        let mut has_cargo_toml = false;

        for f in &parsed.files {
            if f.path.ends_with("Cargo.toml") {
                has_cargo_toml = true;
            }
            if f.path.contains("tests/") || f.path.contains("test_") || f.path.contains("_test") {
                if let Some(name) = std::path::Path::new(&f.path).file_stem() {
                    modules.push(name.to_string_lossy().to_string());
                }
            }
        }

        if has_cargo_toml {
            test_framework = Some("cargo-test".into());
        } else if self.config.language == "python" {
            test_framework = Some("pytest".into());
        }

        self.context.analyzed = Some(AnalyzedOutput {
            modules,
            test_framework,
            has_cargo_toml,
        });
        Ok(())
    }

    pub fn run_generate(&mut self) -> anyhow::Result<()> {
        let root = &self.workspace.root;
        let mut files_written = Vec::new();
        let mut source = GenerationSource::Skipped;

        // 1. Run constraint solver — match patterns from the database
        let solver_result = crate::solver::solve(&self.config);
        for m in &solver_result.matched {
            if let Some(pattern) = solver_result.patterns.iter().find(|p| p.id == m.pattern_id) {
                let vars = crate::solver::template::extract_variables(&pattern.template);
                let mut var_map = std::collections::HashMap::new();
                for v in &vars {
                    var_map.insert(v.clone(), format!("<{v}>"));
                }
                let code = crate::solver::template::apply(pattern, &var_map);
                let filename = format!("{}.rs", pattern.name.replace('-', "_"));
                let output_path = root.join("src").join(&filename);
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&output_path, &code)?;
                files_written.push(filename);
                source = GenerationSource::Solver { pattern_id: pattern.id.clone() };
                // Track pattern usage
                let mut db = crate::solver::pattern_db::PatternDatabase::open();
                let _ = db.increment_usage(&pattern.id);
                break; // only apply the best match
            }
        }

        // 2. Compile .khold spec files
        for entry in walkdir(root, 5, &[".git", "target", ".venv", "node_modules"])? {
            if entry.extension().and_then(|e| e.to_str()) == Some("khold") {
                let output_dir = root.join("src");
                std::fs::create_dir_all(&output_dir)?;
                match crate::dsl::compile_spec(&entry, &output_dir) {
                    Ok(result) => {
                        files_written.extend(result.files);
                        if !matches!(source, GenerationSource::Solver { .. }) {
                            source = GenerationSource::DslCompiled;
                        }
                    }
                    Err(e) => {
                        eprintln!("  ⚠ failed to compile {}: {e}", entry.display());
                    }
                }
            }
        }

        self.context.generated = Some(GeneratedOutput { files_written, source });
        Ok(())
    }

    pub fn run_verify(&mut self) -> anyhow::Result<()> {
        let mut results = Vec::new();

        for cmd in &self.config.pipeline.verify {
            let result = crate::util::run_cmd(cmd, &self.workspace.root);
            results.push(match result.status {
                crate::util::CmdStatus::Ok => PhaseCheck {
                    name: result.name,
                    status: PhaseStatus::Pass,
                    message: result.message,
                },
                crate::util::CmdStatus::Warn => PhaseCheck {
                    name: result.name,
                    status: PhaseStatus::Warn,
                    message: result.message,
                },
                crate::util::CmdStatus::Fail => PhaseCheck {
                    name: result.name,
                    status: PhaseStatus::Fail,
                    message: result.message,
                },
            });
        }

        self.context.verified = Some(VerifiedOutput { results });
        Ok(())
    }

    pub fn run_optimize(&mut self) -> anyhow::Result<()> {
        let mut formatted = Vec::new();

        for cmd in &self.config.pipeline.optimize {
            let result = crate::util::run_cmd(cmd, &self.workspace.root);
            if matches!(result.status, crate::util::CmdStatus::Ok) {
                formatted.push(result.name);
            }
        }

        self.context.optimized = Some(OptimizedOutput {
            formatted_files: formatted,
        });
        Ok(())
    }

    pub fn report(&self, json: bool) -> anyhow::Result<()> {
        if json {
            let mut phases = Vec::new();
            if let Some(ref p) = self.context.parsed {
                let total_size: u64 = p.files.iter().map(|f| f.size_bytes).sum();
                let has_yaml = p.files.iter().any(|f| f.has_kodehold_yaml);
                let mut lang_set: Vec<&str> = p.files.iter().map(|f| f.language.as_str()).collect();
                lang_set.sort();
                lang_set.dedup();
                phases.push(serde_json::json!({
                    "phase": "parse",
                    "status": "ok",
                    "files": p.files.len(),
                    "total_size_bytes": total_size,
                    "has_kodehold_yaml": has_yaml,
                    "languages": lang_set,
                }));
            }
            if let Some(ref a) = self.context.analyzed {
                phases.push(serde_json::json!({
                    "phase": "analyze",
                    "status": "ok",
                    "modules": a.modules.len(),
                    "test_framework": a.test_framework,
                    "has_cargo_toml": a.has_cargo_toml,
                }));
            }
            if let Some(ref g) = self.context.generated {
                let source = match &g.source {
                    GenerationSource::Solver { pattern_id } => format!("solver:{pattern_id}"),
                    GenerationSource::AiFallback => "ai-fallback".into(),
                    GenerationSource::DslCompiled => "dsl-compiled".into(),
                    GenerationSource::Skipped => "skipped".into(),
                };
                phases.push(serde_json::json!({
                    "phase": "generate",
                    "status": "ok",
                    "files_written": g.files_written.len(),
                    "source": source,
                }));
            }
            if let Some(ref v) = self.context.verified {
                let checks: Vec<serde_json::Value> = v.results.iter().map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "status": format!("{:?}", r.status),
                        "message": r.message,
                    })
                }).collect();
                phases.push(serde_json::json!({
                    "phase": "verify",
                    "status": if v.results.iter().any(|r| r.status == PhaseStatus::Fail) { "fail" } else { "ok" },
                    "checks": checks,
                }));
            }
            if let Some(ref o) = self.context.optimized {
                phases.push(serde_json::json!({
                    "phase": "optimize",
                    "status": "ok",
                    "formatted": o.formatted_files.len(),
                }));
            }
            let output = serde_json::json!({ "pipeline": phases });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("━━━ Pipeline ━━━");
            if let Some(ref p) = self.context.parsed {
                let total_size: u64 = p.files.iter().map(|f| f.size_bytes).sum();
                let has_yaml = if p.files.iter().any(|f| f.has_kodehold_yaml) { " +kodehold.yaml" } else { "" };
                let mut lang_set: Vec<&str> = p.files.iter().map(|f| f.language.as_str()).collect();
                lang_set.sort();
                lang_set.dedup();
                println!("  ✓ parse: {} file(s), {} bytes total, langs: {}{has_yaml}",
                    p.files.len(), total_size, lang_set.join(", "));
            }
            if let Some(ref a) = self.context.analyzed {
                let fw = a.test_framework.as_deref().unwrap_or("none");
                let cargo = if a.has_cargo_toml { " (Cargo.toml)" } else { "" };
                println!("  ✓ analyze: {} module(s), framework: {fw}{cargo}", a.modules.len());
            }
            if let Some(ref g) = self.context.generated {
                let src = match &g.source {
                    GenerationSource::Solver { pattern_id } => format!("solver ({pattern_id})"),
                    GenerationSource::AiFallback => "ai-fallback (not yet implemented)".into(),
                    GenerationSource::DslCompiled => "dsl-compiled".into(),
                    GenerationSource::Skipped => "skipped (no patterns defined)".into(),
                };
                println!("  ✓ generate: {} file(s), source: {src}", g.files_written.len());
            }
            if let Some(ref v) = self.context.verified {
                let fail = v.results.iter().filter(|r| r.status == PhaseStatus::Fail).count();
                let pass = v.results.iter().filter(|r| r.status == PhaseStatus::Pass).count();
                if fail > 0 {
                    println!("  ✗ verify: {pass} passed, {fail} failed");
                } else {
                    println!("  ✓ verify: {pass} checks");
                }
            }
            if let Some(ref o) = self.context.optimized {
                println!("  ✓ optimize: {} file(s) formatted", o.formatted_files.len());
            }
        }
        Ok(())
    }
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self {
            parsed: None,
            analyzed: None,
            generated: None,
            verified: None,
            optimized: None,
        }
    }
}

fn walkdir(root: &std::path::Path, max_depth: usize, exclude: &[&str]) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > max_depth {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_dir = path.is_dir();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if exclude.contains(&name) || name.starts_with('.') {
                    continue;
                }
                if is_dir {
                    stack.push((path, depth + 1));
                } else {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}
