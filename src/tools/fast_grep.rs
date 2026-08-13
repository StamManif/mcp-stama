use crate::tools::McpTool;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader};


#[derive(Default)]
pub struct FastGrepTool;

impl FastGrepTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl McpTool for FastGrepTool {
    fn name(&self) -> &'static str {
        "fast_grep"
    }

    fn description(&self) -> &'static str {
        "Ultra-fast file search and regex grepping respecting .gitignore rules."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search string or regex pattern."
                },
                "path": {
                    "type": "string",
                    "description": "Root directory to search from.",
                    "default": "."
                },
                "extension": {
                    "type": "string",
                    "description": "Restrict search to files with this extension (e.g., 'rs', 'ts')."
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of matched lines to return (default: 100).",
                    "default": 100
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .context("Missing required parameter 'query'")?;

        let search_path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let target_ext = args
            .get("extension")
            .and_then(|v| v.as_str())
            .map(|e| e.trim_start_matches('.'));

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let re = RegexBuilder::new(query)
            .build()
            .with_context(|| format!("Invalid regex pattern: '{}'", query))?;

        let mut matches = Vec::new();
        let mut truncated = false;

        let walker = WalkBuilder::new(search_path)
            .hidden(true)
            .git_ignore(true)
            .ignore(true)
            .parents(true)
            .require_git(false)
            .build();

        'outer: for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            // Only process regular files
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                continue;
            }

            let path = entry.path();

            // Filter by extension if specified
            if let Some(ext) = target_ext {
                match path.extension().and_then(|e| e.to_str()) {
                    Some(file_ext) if file_ext.eq_ignore_ascii_case(ext) => {}
                    _ => continue,
                }
            }

            // Attempt to open the file
            let file = match File::open(path) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let reader = BufReader::new(file);
            let display_path = path.to_string_lossy().to_string();

            for (line_idx, line_res) in reader.lines().enumerate() {
                let line_text = match line_res {
                    Ok(l) => l,
                    Err(_) => break, // Skip unreadable / binary files gracefully
                };

                if re.is_match(&line_text) {
                    matches.push(json!({
                        "file": display_path,
                        "line_number": line_idx + 1,
                        "line_text": line_text
                    }));

                    if matches.len() >= max_results {
                        truncated = true;
                        break 'outer;
                    }
                }
            }
        }

        let matches_count = matches.len();

        Ok(json!({
            "matches_count": matches_count,
            "truncated": truncated,
            "matches": matches
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_fast_grep_schema() {
        let tool = FastGrepTool::new();
        assert_eq!(tool.name(), "fast_grep");
        let schema = tool.schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["query"].is_object());
    }

    #[tokio::test]
    async fn test_fast_grep_search() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("test1.rs");
        let file2_path = dir.path().join("test2.txt");

        fs::write(&file1_path, "fn greet() {\n    println!(\"hello\");\n}\n").unwrap();
        fs::write(&file2_path, "another line\nhello from text file\n").unwrap();

        let tool = FastGrepTool::new();
        let args = json!({
            "query": "hello",
            "path": dir.path().to_str().unwrap(),
            "max_results": 10
        });

        let res = tool.execute(args).await.unwrap();
        assert_eq!(res["matches_count"], 2);
        assert_eq!(res["truncated"], false);
        assert_eq!(res["matches"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_fast_grep_gitignore() {
        let dir = tempdir().unwrap();
        let gitignore_path = dir.path().join(".gitignore");
        let ignored_file = dir.path().join("secret.log");
        let tracked_file = dir.path().join("app.rs");

        fs::write(&gitignore_path, "*.log\n").unwrap();
        fs::write(&ignored_file, "TARGET_SECRET_PATTERN\n").unwrap();
        fs::write(&tracked_file, "TARGET_SECRET_PATTERN in code\n").unwrap();

        let tool = FastGrepTool::new();
        let args = json!({
            "query": "TARGET_SECRET_PATTERN",
            "path": dir.path().to_str().unwrap()
        });

        let res = tool.execute(args).await.unwrap();
        assert_eq!(res["matches_count"], 1);
        let matched_file = res["matches"][0]["file"].as_str().unwrap();
        assert!(matched_file.contains("app.rs"));
        assert!(!matched_file.contains("secret.log"));
    }
}

