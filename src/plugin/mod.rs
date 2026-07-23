use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookConfig {
    pub pre_generate: Option<String>,
    pub post_verify: Option<String>,
    pub on_ship: Option<String>,
}

#[derive(Debug)]
pub struct HookResult {
    pub hook: String,
    pub status: HookStatus,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum HookStatus {
    Ok,
    Blocked,
    Skipped,
}

impl HookConfig {
    pub fn from_config(config: &Config) -> Self {
        config.hooks.clone()
    }

    pub fn list_available(&self, workspace_root: &std::path::Path) -> Vec<String> {
        let mut hooks = Vec::new();
        for (name, path) in self.iter() {
            if let Some(p) = path {
                let full_path = workspace_root.join(p);
                if full_path.exists() {
                    hooks.push(name.to_string());
                }
            }
        }
        hooks
    }
}

pub fn run_hook(workspace_root: &std::path::Path, hook_path: &str) -> HookResult {
    let full_path = workspace_root.join(hook_path);

    if !full_path.exists() {
        return HookResult {
            hook: hook_path.to_string(),
            status: HookStatus::Skipped,
            message: "hook not found".into(),
        };
    }

    let payload = serde_json::json!({
        "workspace": workspace_root.to_string_lossy(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let output = std::process::Command::new(&full_path)
        .arg(serde_json::to_string(&payload).unwrap_or_default())
        .current_dir(workspace_root)
        .output();

    match output {
        Ok(out) if out.status.success() => HookResult {
            hook: hook_path.to_string(),
            status: HookStatus::Ok,
            message: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        },
        Ok(out) => HookResult {
            hook: hook_path.to_string(),
            status: HookStatus::Blocked,
            message: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        },
        Err(e) => HookResult {
            hook: hook_path.to_string(),
            status: HookStatus::Skipped,
            message: format!("failed to execute: {e}"),
        },
    }
}

/// Scaffold a new hook script
pub fn scaffold_hook(workspace_root: &std::path::Path, name: &str, language: &str) -> anyhow::Result<String> {
    let hooks_dir = workspace_root.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let path = hooks_dir.join(name);
    let content = match language {
        "bash" | "sh" => format!(
            r##"#!/bin/bash
# KodeHold hook: {name}
echo '{{"status":"ok"}}'"##
        ),
        "python" => format!(
            r##"#!/usr/bin/env python3
import sys, json
payload = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {{}}
print(json.dumps({{"status":"ok"}}))
"##
        ),
        _ => anyhow::bail!("unsupported hook language: {language}"),
    };

    std::fs::write(&path, content)?;
    std::process::Command::new("chmod").arg("+x").arg(&path).status()?;
    Ok(path.to_string_lossy().to_string())
}

impl HookConfig {
    fn iter(&self) -> Vec<(&str, Option<&str>)> {
        vec![
            ("pre-generate", self.pre_generate.as_deref()),
            ("post-verify", self.post_verify.as_deref()),
            ("on-ship", self.on_ship.as_deref()),
        ]
    }
}
