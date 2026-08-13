use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub fn inject_mcp_server_config(config_path: &Path, exe_path: &Path) -> Result<()> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create directory {:?}", parent))?;
    }

    let mut doc: Value = if config_path.exists() {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file at {:?}", config_path))?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    if !doc.is_object() {
        doc = json!({});
    }

    if doc.get("mcpServers").is_none() || !doc["mcpServers"].is_object() {
        doc["mcpServers"] = json!({});
    }

    let exe_str = exe_path.to_string_lossy().to_string();
    doc["mcpServers"]["mcp-stama"] = json!({
        "command": exe_str,
        "args": []
    });

    let formatted = serde_json::to_string_pretty(&doc)?;
    fs::write(config_path, formatted)
        .with_context(|| format!("Failed to write config file to {:?}", config_path))?;

    Ok(())
}

pub fn run_installer(install_cursor: bool, install_claude: bool) -> Result<()> {
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    if install_cursor {
        let cursor_path = get_cursor_config_path()?;
        inject_mcp_server_config(&cursor_path, &current_exe)?;
        eprintln!(
            "Successfully installed mcp-stama to Cursor configuration at: {:?}",
            cursor_path
        );
    }

    if install_claude {
        let claude_path = get_claude_config_path()?;
        inject_mcp_server_config(&claude_path, &current_exe)?;
        eprintln!(
            "Successfully installed mcp-stama to Claude Desktop configuration at: {:?}",
            claude_path
        );
    }

    Ok(())
}

fn get_cursor_config_path() -> Result<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("Could not determine user home directory")?;
    Ok(PathBuf::from(home).join(".cursor").join("mcp.json"))
}

fn get_claude_config_path() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .or_else(|_| std::env::var("USERPROFILE").map(|h| format!("{}\\AppData\\Roaming", h)))
            .context("Could not determine APPDATA directory")?;
        Ok(PathBuf::from(appdata)
            .join("Claude")
            .join("claude_desktop_config.json"))
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").context("Could not determine HOME directory")?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json"))
    } else {
        let home = std::env::var("HOME").context("Could not determine HOME directory")?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("Claude")
            .join("claude_desktop_config.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_installer_injection_new_file() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("mcp.json");
        let mock_exe = PathBuf::from("C:\\tools\\mcp-stama.exe");

        inject_mcp_server_config(&config_file, &mock_exe).unwrap();

        let content = fs::read_to_string(&config_file).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(
            parsed["mcpServers"]["mcp-stama"]["command"].as_str().unwrap(),
            mock_exe.to_string_lossy().as_ref()
        );
        assert_eq!(
            parsed["mcpServers"]["mcp-stama"]["args"].as_array().unwrap().len(),
            0
        );
    }

    #[test]
    fn test_installer_injection_existing_file() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("claude_desktop_config.json");
        let initial_json = json!({
            "mcpServers": {
                "other_tool": {
                    "command": "other_exe",
                    "args": ["--flag"]
                }
            }
        });

        fs::write(&config_file, serde_json::to_string_pretty(&initial_json).unwrap()).unwrap();

        let mock_exe = PathBuf::from("/usr/local/bin/mcp-stama");
        inject_mcp_server_config(&config_file, &mock_exe).unwrap();

        let content = fs::read_to_string(&config_file).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["mcpServers"]["other_tool"]["command"], "other_exe");
        assert_eq!(
            parsed["mcpServers"]["mcp-stama"]["command"].as_str().unwrap(),
            mock_exe.to_string_lossy().as_ref()
        );
    }

}
