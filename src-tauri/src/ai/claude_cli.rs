//! Bridge to a locally installed, logged-in Claude Code CLI (`claude -p`).
//! Uses the user's own Claude subscription on their own machine.

use std::path::PathBuf;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use super::{AiRequest, AiResponse, timeout};
use crate::config::AiProfile;
use crate::error::{AppError, AppResult};

/// Finds the `claude` executable: explicit path, PATH, then common install
/// locations of the native installer and npm.
pub fn detect(custom: &str) -> Option<PathBuf> {
    let custom = custom.trim();
    if !custom.is_empty() {
        let p = PathBuf::from(custom);
        return p.exists().then_some(p);
    }
    let names: &[&str] = if cfg!(windows) { &["claude.exe", "claude.cmd"] } else { &["claude"] };
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for n in names {
                let candidate = dir.join(n);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)?;
    let mut candidates = vec![home.join(".local").join("bin").join(names[0]), home.join(".claude").join("local").join(names[0])];
    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            candidates.push(PathBuf::from(appdata).join("npm").join("claude.cmd"));
        }
    } else {
        candidates.push(PathBuf::from("/usr/local/bin/claude"));
        candidates.push(PathBuf::from("/opt/homebrew/bin/claude"));
    }
    candidates.into_iter().find(|c| c.is_file()).or_else(|| desktop_bundled(&home, names[0]))
}

/// The Claude desktop app ships its own CLI under
/// `<config>/Claude/claude-code/<version>/<build>/claude(.exe)`; picks the newest.
/// On Windows the Store/MSIX build keeps it in its package folder.
fn desktop_bundled(home: &std::path::Path, exe: &str) -> Option<PathBuf> {
    let mut roots = Vec::new();
    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            roots.push(PathBuf::from(appdata).join("Claude"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let packages = PathBuf::from(local).join("Packages");
            for pkg in std::fs::read_dir(packages).into_iter().flatten().flatten() {
                if pkg.file_name().to_string_lossy().starts_with("Claude_") {
                    roots.push(pkg.path().join("LocalCache").join("Roaming").join("Claude"));
                }
            }
        }
    } else if cfg!(target_os = "macos") {
        roots.push(home.join("Library").join("Application Support").join("Claude"));
    } else {
        roots.push(home.join(".config").join("Claude"));
    }
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for root in roots {
        for version in std::fs::read_dir(root.join("claude-code")).into_iter().flatten().flatten() {
            for build in std::fs::read_dir(version.path()).into_iter().flatten().flatten() {
                let candidate = build.path().join(exe);
                if let Ok(meta) = candidate.metadata()
                    && meta.is_file() {
                        found.push((meta.modified().unwrap_or(std::time::UNIX_EPOCH), candidate));
                    }
            }
        }
    }
    found.into_iter().max_by_key(|(t, _)| *t).map(|(_, p)| p)
}

pub async fn complete(p: &AiProfile, req: &AiRequest) -> AppResult<AiResponse> {
    let exe = detect(&p.cli_path).ok_or_else(|| {
        AppError::msg("Claude Code CLI not found. Install it and log in with `claude`, or set its path in the AI profile.")
    })?;

    // The system prompt can be long and multi-line, so pass it as a file.
    let prompt_file = std::env::temp_dir().join(format!("highbazar-system-{}.txt", uuid::Uuid::new_v4().simple()));
    tokio::fs::write(&prompt_file, &req.system).await?;

    let is_cmd = exe.extension().is_some_and(|e| e.eq_ignore_ascii_case("cmd"));
    let mut cmd = if is_cmd {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(&exe);
        c
    } else {
        Command::new(&exe)
    };
    cmd.arg("-p")
        .arg("--output-format")
        .arg("json")
        .arg("--system-prompt-file")
        .arg(&prompt_file)
        .arg("--tools")
        .arg("")
        .arg("--no-session-persistence")
        .arg("--max-turns")
        .arg("1");
    if !p.model.trim().is_empty() {
        cmd.arg("--model").arg(p.model.trim());
    }
    if !p.reasoning.trim().is_empty() {
        cmd.arg("--effort").arg(p.reasoning.trim());
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let result = async {
        let mut child = cmd.spawn().map_err(|e| AppError::msg(format!("Could not start Claude Code CLI: {e}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(req.user.as_bytes()).await?;
            drop(stdin);
        }
        let out = tokio::time::timeout(timeout(p), child.wait_with_output())
            .await
            .map_err(|_| AppError::msg("Claude Code CLI timed out"))??;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let v: Value = serde_json::from_str(stdout.trim()).map_err(|_| {
            let err = String::from_utf8_lossy(&out.stderr);
            AppError::msg(format!(
                "Claude Code CLI failed: {}",
                if err.trim().is_empty() { stdout.chars().take(300).collect::<String>() } else { err.chars().take(300).collect() }
            ))
        })?;
        parse(&v).map_err(|e| {
            if e.to_string().to_lowercase().contains("not logged in") {
                AppError::msg(format!(
                    "Claude Code CLI is not logged in. Open a terminal, run \"{}\" and type /login once.",
                    exe.display()
                ))
            } else {
                e
            }
        })
    }
    .await;
    let _ = tokio::fs::remove_file(&prompt_file).await;
    result
}

pub fn parse(v: &Value) -> AppResult<AiResponse> {
    if v["is_error"].as_bool().unwrap_or(false) {
        return Err(AppError::msg(format!(
            "Claude Code CLI error: {}",
            v["result"].as_str().or(v["subtype"].as_str()).unwrap_or("unknown")
        )));
    }
    let u = &v["usage"];
    Ok(AiResponse {
        text: v["result"].as_str().unwrap_or_default().to_string(),
        input_tokens: u["input_tokens"].as_u64().unwrap_or(0)
            + u["cache_read_input_tokens"].as_u64().unwrap_or(0)
            + u["cache_creation_input_tokens"].as_u64().unwrap_or(0),
        output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
        // Covered by the subscription – does not count against the API budget.
        reported_cost_usd: Some(0.0),
        tool_cost_usd: 0.0,
        model: "claude-code".into(),
    })
}
