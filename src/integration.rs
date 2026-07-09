#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

use crate::config::{Config, config_dir, ensure_parent};

const SHIM_MARKER: &str = "openclaude claude autostart shim";

#[derive(Debug, Serialize, Deserialize)]
struct ShimState {
    wrapper_path: PathBuf,
    original_path: PathBuf,
    backup_path: PathBuf,
}

pub fn shim_state_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("claude-shim.json"))
}

pub fn install_claude_shim() -> anyhow::Result<()> {
    let cfg = Config::load_or_create()?;
    let claude = find_on_path("claude").context("could not find claude on PATH")?;
    if is_shim(&claude) {
        if let Ok(raw) = fs::read_to_string(shim_state_path()?) {
            if let Ok(state) = serde_json::from_str::<ShimState>(&raw) {
                let occ = env::current_exe().context("resolve current occ executable")?;
                write_unix_shim(&claude, &state.backup_path, &occ, &cfg)?;
                println!("refreshed claude shim: {}", claude.display());
                return Ok(());
            }
        }
        println!("claude shim already installed: {}", claude.display());
        return Ok(());
    }
    let backup = backup_path_for(&claude);
    if backup.exists() {
        bail!(
            "backup already exists at {}; refusing to overwrite",
            backup.display()
        );
    }
    fs::rename(&claude, &backup).with_context(|| format!("backup {}", claude.display()))?;
    let occ = env::current_exe().context("resolve current occ executable")?;
    write_unix_shim(&claude, &backup, &occ, &cfg)?;
    let state = ShimState {
        wrapper_path: claude.clone(),
        original_path: claude,
        backup_path: backup,
    };
    let state_path = shim_state_path()?;
    ensure_parent(&state_path)?;
    fs::write(&state_path, serde_json::to_string_pretty(&state)? + "\n")?;
    println!("installed claude shim. Use `occ native` to restore native Claude Code.");
    Ok(())
}

pub fn uninstall_claude_shim() -> anyhow::Result<()> {
    let state_path = shim_state_path()?;
    if !state_path.exists() {
        println!("no openclaude claude shim state found; native mode already active");
        return Ok(());
    }
    let state: ShimState = serde_json::from_str(&fs::read_to_string(&state_path)?)?;
    if state.wrapper_path.exists() && is_shim(&state.wrapper_path) {
        fs::remove_file(&state.wrapper_path)?;
    }
    if state.backup_path.exists() {
        fs::rename(&state.backup_path, &state.original_path)?;
    }
    fs::remove_file(state_path)?;
    println!("restored native Claude Code launcher.");
    Ok(())
}

fn find_on_path(command: &str) -> Option<PathBuf> {
    for dir in env::split_paths(&env::var_os("PATH")?) {
        let path = dir.join(command);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn backup_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("claude");
    path.with_file_name(format!("{file_name}.openclaude-real"))
}

fn is_shim(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|s| s.contains(SHIM_MARKER))
        .unwrap_or(false)
}

fn shell_quote(value: &Path) -> String {
    let raw = value.to_string_lossy();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn write_unix_shim(
    wrapper: &Path,
    real_claude: &Path,
    occ: &Path,
    cfg: &Config,
) -> anyhow::Result<()> {
    #[cfg(not(unix))]
    {
        let _ = (wrapper, real_claude, occ, cfg);
        anyhow::bail!("claude shim is currently implemented for Unix-like systems only");
    }
    #[cfg(unix)]
    {
        let model_env = cfg
            .claude_model_env()
            .into_iter()
            .map(|(name, value)| format!("export {name}={}\n", shell_quote_str(&value)))
            .collect::<String>();
        let body = format!(
            r#"#!/usr/bin/env sh
# {SHIM_MARKER}
{occ} ensure >/dev/null 2>&1 || true
export ANTHROPIC_BASE_URL="http://{host}:{port}"
unset ANTHROPIC_AUTH_TOKEN
export ANTHROPIC_API_KEY="{token}"
export CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1
{model_env}
exec {real} "$@"
"#,
            occ = shell_quote(occ),
            host = cfg.host,
            port = cfg.port,
            token = cfg.gateway_token,
            model_env = model_env,
            real = shell_quote(real_claude),
        );
        fs::write(wrapper, body)?;
        let mut perms = fs::metadata(wrapper)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(wrapper, perms)?;
        Ok(())
    }
}
