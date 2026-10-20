//! Bridge to a locally installed, logged-in OpenAI Codex CLI (`codex exec`).
//! Uses the user's own ChatGPT plan (or the API key Codex is logged in with)
//! on their own machine.
//!
//! The bot only needs a text answer, so everything else Codex can do is shut
//! off: no shell or code tools, a read-only sandbox, no user config, rules,
//! project instructions or saved sessions, and Codex's own coding-agent
//! instructions are replaced by HighBazar's system prompt.

use std::path::{Path, PathBuf};

use serde_json::Value;
use tokio::io::AsyncWriteExt;

use super::claude_cli::command_for;
use super::{AiRequest, AiResponse, ModelInfo, timeout};
use crate::config::AiProfile;
use crate::error::{AppError, AppResult};

const TRIPLES: &[&str] = if cfg!(windows) {
    &["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"]
} else if cfg!(target_os = "macos") {
    &["aarch64-apple-darwin", "x86_64-apple-darwin"]
} else {
    &["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
};

/// Finds the `codex` executable: explicit path, PATH, then the places the
/// official installers use. An npm launcher is resolved to its native binary.
pub fn detect(custom: &str) -> Option<PathBuf> {
    let custom = custom.trim();
    if !custom.is_empty() {
        let p = PathBuf::from(custom);
        return p.is_file().then(|| native_binary(&p).unwrap_or(p));
    }
    let names: &[&str] = if cfg!(windows) { &["codex.exe", "codex.cmd"] } else { &["codex"] };
    let mut candidates = Vec::new();
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            candidates.extend(names.iter().map(|n| dir.join(n)));
        }
    }
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    if cfg!(windows) {
        // Standalone installer (install.ps1).
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(PathBuf::from(local).join("Programs").join("OpenAI").join("Codex").join("bin").join("codex.exe"));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            candidates.push(PathBuf::from(appdata).join("npm").join("codex.cmd"));
        }
    } else {
        if let Some(h) = &home {
            candidates.push(h.join(".local").join("bin").join("codex"));
        }
        candidates.push(PathBuf::from("/opt/homebrew/bin/codex"));
        candidates.push(PathBuf::from("/usr/local/bin/codex"));
    }
    let found = candidates.into_iter().find(|c| c.is_file())?;
    Some(native_binary(&found).unwrap_or(found))
}

/// The npm package starts the real binary through Node; calling it directly
/// saves a process and avoids `cmd.exe` quoting.
fn native_binary(launcher: &Path) -> Option<PathBuf> {
    let is_launcher = launcher.extension().is_some_and(|e| e.eq_ignore_ascii_case("cmd")) || launcher.is_symlink();
    if !is_launcher {
        return None;
    }
    let launcher = std::fs::canonicalize(launcher).unwrap_or_else(|_| launcher.to_path_buf());
    let exe = if cfg!(windows) { "codex.exe" } else { "codex" };
    // <prefix>/codex.cmd → <prefix>/node_modules/@openai/codex/…; a symlink
    // points into <pkg>/bin/codex.js.
    let mut roots = Vec::new();
    for dir in launcher.ancestors().skip(1).take(4) {
        roots.push(dir.join("node_modules").join("@openai").join("codex"));
        if dir.file_name().is_some_and(|n| n == "codex") {
            roots.push(dir.to_path_buf());
        }
    }
    for root in roots {
        for triple in TRIPLES {
            let platform = platform_package(triple);
            for vendor in [
                root.join("node_modules").join("@openai").join(platform).join("vendor"),
                root.parent().map(|p| p.join(platform).join("vendor")).unwrap_or_default(),
                root.join("vendor"),
            ] {
                let candidate = vendor.join(triple).join("bin").join(exe);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn platform_package(triple: &str) -> &'static str {
    match triple {
        "x86_64-pc-windows-msvc" => "codex-win32-x64",
        "aarch64-pc-windows-msvc" => "codex-win32-arm64",
        "aarch64-apple-darwin" => "codex-darwin-arm64",
        "x86_64-apple-darwin" => "codex-darwin-x64",
        "aarch64-unknown-linux-musl" => "codex-linux-arm64",
        _ => "codex-linux-x64",
    }
}

fn find(p: &AiProfile) -> AppResult<PathBuf> {
    detect(&p.cli_path).ok_or_else(|| {
        AppError::msg("Codex CLI not found. Install it and log in with `codex login`, or set its path in the AI profile.")
    })
}

async fn output(mut cmd: tokio::process::Command, stdin: Option<&str>, limit: std::time::Duration) -> AppResult<std::process::Output> {
    let mut child = cmd.spawn().map_err(|e| AppError::msg(format!("Could not start Codex CLI: {e}")))?;
    if let Some(mut pipe) = child.stdin.take() {
        if let Some(text) = stdin {
            pipe.write_all(text.as_bytes()).await?;
        }
        drop(pipe);
    }
    tokio::time::timeout(limit, child.wait_with_output())
        .await
        .map_err(|_| AppError::msg("Codex CLI timed out"))?
        .map_err(Into::into)
}

/// "Logged in using ChatGPT" etc.; `None` when not logged in.
pub async fn login_status(path: &str) -> AppResult<Option<String>> {
    let exe = detect(path).ok_or_else(|| AppError::msg("Codex CLI not found."))?;
    let mut cmd = command_for(&exe);
    cmd.arg("login").arg("status");
    let out = output(cmd, None, std::time::Duration::from_secs(20)).await?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let line = text.lines().map(str::trim).find(|l| l.to_lowercase().contains("logged in")).unwrap_or("").to_string();
    Ok(out.status.success().then_some(line))
}

pub async fn list_models(p: &AiProfile) -> AppResult<Vec<ModelInfo>> {
    let exe = find(p)?;
    let mut cmd = command_for(&exe);
    cmd.arg("debug").arg("models");
    let out = output(cmd, None, std::time::Duration::from_secs(30)).await?;
    let v: Value = serde_json::from_slice(&out.stdout)
        .map_err(|_| AppError::msg(format!("Codex CLI did not list its models: {}", String::from_utf8_lossy(&out.stderr).trim())))?;
    Ok(parse_models(&v))
}

pub fn parse_models(v: &Value) -> Vec<ModelInfo> {
    v["models"]
        .as_array()
        .into_iter()
        .flatten()
        // Hidden entries are internal (review/special access models).
        .filter(|m| m["visibility"].as_str().unwrap_or("list") == "list")
        .filter_map(|m| {
            let id = m["slug"].as_str()?;
            Some(ModelInfo::new(id, m["display_name"].as_str().unwrap_or_default()))
        })
        .collect()
}

pub async fn complete(p: &AiProfile, req: &AiRequest) -> AppResult<AiResponse> {
    let exe = find(p)?;
    // Replaces Codex's built-in instructions, so it is no coding agent here.
    let prompt_file = std::env::temp_dir().join(format!("highbazar-codex-{}.md", uuid::Uuid::new_v4().simple()));
    tokio::fs::write(&prompt_file, &req.system).await?;
    let result = run(&exe, p, req, &prompt_file).await;
    let _ = tokio::fs::remove_file(&prompt_file).await;
    result
}

/// The `codex exec` arguments (prompt from stdin).
fn args(p: &AiProfile, req: &AiRequest, prompt_file: &Path, workdir: &Path) -> Vec<String> {
    let mut a: Vec<String> = [
        "exec",
        "--json",
        "--ephemeral",
        "--skip-git-repo-check",
        "--ignore-user-config",
        "--ignore-rules",
        "--color",
        "never",
        "--sandbox",
        "read-only",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    // Feature switches go through -c: an unknown name (older or newer Codex)
    // is only ignored there, while `--disable <name>` would abort the run.
    const OVERRIDES: &[&str] = &[
        // No command, code, image or agent tools; verified against codex-cli 0.163.0.
        "features.shell_tool=false",
        "features.unified_exec=false",
        "features.code_mode_host=false",
        "features.multi_agent=false",
        "features.sleep_tool=false",
        "features.skill_search=false",
        "features.apps=false",
        "features.plugins=false",
        "features.image_generation=false",
        "features.browser_use=false",
        "features.computer_use=false",
        "agents.enabled=false",
        "skills.max_context_tokens=1",
        "include_permissions_instructions=false",
        "include_environment_context=false",
        "project_doc_max_bytes=0",
        "hide_agent_reasoning=true",
    ];
    for o in OVERRIDES {
        a.push("-c".into());
        a.push((*o).into());
    }
    // Values that are no valid TOML are taken literally, so no quoting is needed.
    a.push("-c".into());
    a.push(format!("web_search={}", if req.web_search { "live" } else { "disabled" }));
    a.push("-c".into());
    a.push(format!("model_instructions_file={}", serde_json::to_string(&prompt_file.to_string_lossy()).unwrap_or_default()));
    if !p.model.trim().is_empty() {
        a.push("--model".into());
        a.push(p.model.trim().to_string());
    }
    if !p.reasoning.trim().is_empty() {
        a.push("-c".into());
        a.push(format!("model_reasoning_effort={}", p.reasoning.trim()));
    }
    a.push("--cd".into());
    a.push(workdir.to_string_lossy().to_string());
    a.push("-".into());
    a
}

async fn run(exe: &Path, p: &AiProfile, req: &AiRequest, prompt_file: &Path) -> AppResult<AiResponse> {
    let mut cmd = command_for(exe);
    cmd.args(args(p, req, prompt_file, &std::env::temp_dir()));
    let out = output(cmd, Some(&req.user), timeout(p)).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    parse(&stdout).map_err(|e| {
        let m = e.to_string();
        let lower = m.to_lowercase();
        if lower.contains("401") || lower.contains("not logged in") || lower.contains("unauthorized") {
            AppError::msg(format!("Codex CLI is not logged in. Open a terminal and run \"{}\" login once.", exe.display()))
        } else if m.contains("no output") {
            let err = String::from_utf8_lossy(&out.stderr);
            let err = err.lines().filter(|l| !l.contains("PATH aliases")).collect::<Vec<_>>().join(" ");
            AppError::msg(format!("Codex CLI failed: {}", err.trim().chars().take(300).collect::<String>()))
        } else {
            e
        }
    })
}

/// Reads the JSONL event stream of `codex exec --json`.
pub fn parse(jsonl: &str) -> AppResult<AiResponse> {
    let mut text: Option<String> = None;
    let mut resp = AiResponse { model: "codex".into(), ..Default::default() };
    let mut error: Option<String> = None;
    let mut any = false;
    for line in jsonl.lines().map(str::trim).filter(|l| l.starts_with('{')) {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        any = true;
        match v["type"].as_str().unwrap_or_default() {
            "item.completed" => {
                let item = &v["item"];
                match item["type"].as_str().unwrap_or_default() {
                    "agent_message" => text = item["text"].as_str().map(str::to_string),
                    kind if kind.contains("web_search") => resp.searches += 1,
                    _ => {}
                }
            }
            "turn.completed" => {
                let u = &v["usage"];
                resp.input_tokens += u["input_tokens"].as_u64().unwrap_or(0);
                resp.output_tokens += u["output_tokens"].as_u64().unwrap_or(0);
            }
            "turn.failed" => error = v["error"]["message"].as_str().map(str::to_string).or(error),
            // Retries are reported as errors too; only the last one counts.
            "error" => error = v["message"].as_str().map(str::to_string),
            _ => {}
        }
    }
    match (text, error) {
        (Some(t), _) => {
            resp.text = t.trim().to_string();
            Ok(resp)
        }
        (None, Some(e)) => Err(AppError::msg(format!("Codex CLI error: {}", e.chars().take(300).collect::<String>()))),
        (None, None) if any => Err(AppError::msg("Codex CLI returned no answer.")),
        (None, None) => Err(AppError::msg("Codex CLI gave no output.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_event_stream() {
        let s = r#"{"type":"thread.started","thread_id":"t1"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"web_search","query":"elden ring patch"}}
{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"  Patch 1.04 is out!  "}}
{"type":"turn.completed","usage":{"input_tokens":2400,"cached_input_tokens":2000,"output_tokens":30,"reasoning_output_tokens":10}}"#;
        let r = parse(s).unwrap();
        assert_eq!(r.text, "Patch 1.04 is out!");
        assert_eq!((r.input_tokens, r.output_tokens, r.searches), (2400, 30, 1));
    }

    #[test]
    fn reports_the_final_error() {
        let s = r#"{"type":"thread.started","thread_id":"t1"}
{"type":"error","message":"Reconnecting... 2/5 (unexpected status 401 Unauthorized)"}
{"type":"turn.failed","error":{"message":"unexpected status 401 Unauthorized: Missing bearer"}}"#;
        let e = parse(s).unwrap_err().to_string();
        assert!(e.contains("401"), "{e}");
        assert!(parse("").unwrap_err().to_string().contains("no output"));
    }

    #[test]
    fn lists_only_visible_models() {
        let v = serde_json::json!({ "models": [
            { "slug": "gpt-6-sol", "display_name": "GPT-6-Sol", "visibility": "list" },
            { "slug": "codex-auto-review", "display_name": "Codex Auto Review", "visibility": "hide" }
        ]});
        assert_eq!(parse_models(&v), vec![ModelInfo::new("gpt-6-sol", "GPT-6-Sol")]);
    }

    #[test]
    fn locks_codex_down() {
        let p = AiProfile { model: "gpt-6-luna".into(), reasoning: "low".into(), ..Default::default() };
        let req = AiRequest { web_search: true, ..Default::default() };
        let a = args(&p, &req, Path::new(r"C:\Temp\hb.md"), Path::new(r"C:\Temp"));
        let joined = a.join(" ");
        for must in ["--sandbox read-only", "features.shell_tool=false", "features.unified_exec=false", "features.code_mode_host=false", "agents.enabled=false", "--ignore-user-config", "--ephemeral", "web_search=live", "model_reasoning_effort=low", "--model gpt-6-luna"] {
            assert!(joined.contains(must), "missing {must}: {joined}");
        }
        assert!(joined.contains(r#"model_instructions_file="C:\\Temp\\hb.md""#), "{joined}");
        assert_eq!(a.last().map(String::as_str), Some("-"));
        let off = args(&p, &AiRequest::default(), Path::new("x"), Path::new("y")).join(" ");
        assert!(off.contains("web_search=disabled"));
    }
}
