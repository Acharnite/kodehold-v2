use crate::cli::VerifyArgs;
use crate::workspace::Workspace;

pub fn run(args: VerifyArgs) -> anyhow::Result<()> {
    let ws = Workspace::open(args.path.as_deref())?;
    let results = ws.verify()?;

    if args.json {
        let checks: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "status": format!("{:?}", r.status),
                    "message": r.message,
                })
            })
            .collect();
        let failed = results.iter().filter(|r| matches!(r.status, crate::workspace::CheckStatus::Fail)).count();
        let passed = results.iter().filter(|r| matches!(r.status, crate::workspace::CheckStatus::Pass)).count();
        let output = serde_json::json!({
            "status": if failed > 0 { "fail" } else { "pass" },
            "checks": checks,
            "summary": { "passed": passed, "failed": failed, "total": results.len() },
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        let mut passed = 0;
        let mut failed = 0;

        for r in &results {
            let icon = match r.status {
                crate::workspace::CheckStatus::Pass => { passed += 1; "✓" }
                crate::workspace::CheckStatus::Warn => { passed += 1; "⚠" }
                crate::workspace::CheckStatus::Fail => { failed += 1; "✗" }
            };
            println!("  {icon} {}: {}", r.name, r.message);
        }

        println!();
        if failed > 0 {
            println!("  ━━━ VERIFICATION FAILED ({passed} passed, {failed} failed) ━━━");
            std::process::exit(1);
        } else {
            println!("  ━━━ VERIFICATION PASSED ({passed} checks) ━━━");
        }
    }

    Ok(())
}
