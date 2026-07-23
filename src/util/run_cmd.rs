#[derive(Debug)]
pub struct CmdResult {
    pub name: String,
    pub status: CmdStatus,
    pub message: String,
}

#[derive(Debug)]
pub enum CmdStatus {
    Ok,
    Warn,
    Fail,
}

pub fn run_cmd(cmd: &str, cwd: &std::path::Path) -> CmdResult {
    let name = cmd.split_whitespace().next().unwrap_or(cmd).to_string();
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(cwd)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            CmdResult {
                name,
                status: CmdStatus::Ok,
                message: if stdout.is_empty() { "passed".into() } else { truncate(&stdout, 200) },
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            CmdResult {
                name,
                status: CmdStatus::Fail,
                message: format!("exit {}: {}", out.status.code().unwrap_or(-1), truncate(&stderr, 200)),
            }
        }
        Err(e) => CmdResult {
            name,
            status: CmdStatus::Fail,
            message: format!("failed to execute: {e}"),
        },
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
