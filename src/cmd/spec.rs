use crate::cli::SpecArgs;
use crate::dsl;

pub fn run(args: SpecArgs) -> anyhow::Result<()> {
    let spec_path = std::path::Path::new(&args.path);
    if !spec_path.exists() {
        anyhow::bail!("spec file not found: {}", args.path);
    }

    let output_dir = args
        .output
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            spec_path
                .parent()
                .map(|p| p.join("src"))
                .unwrap_or_else(|| std::path::PathBuf::from("src"))
        });
    std::fs::create_dir_all(&output_dir)?;

    let result = dsl::compile_spec(spec_path, &output_dir)?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "ok",
                "module": result.module_name,
                "language": result.language,
                "files": result.files,
                "summary": {
                    "types": result.types,
                    "functions": result.functions,
                    "interfaces": result.interfaces,
                    "tests": result.tests,
                }
            }))?
        );
    } else {
        println!("━ Compiling {}", spec_path.display());
        println!("  Module: {} ({})", result.module_name, result.language);
        println!();
        println!("  Generated files:");
        for f in &result.files {
            println!("    {f}");
        }
        println!();
        println!("  Summary:");
        println!("    {} type(s)", result.types);
        println!("    {} function(s)", result.functions);
        println!("    {} interface(s)", result.interfaces);
        println!("    {} test(s)", result.tests);
    }
    Ok(())
}
