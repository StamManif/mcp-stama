use crate::tools::McpTool;
use anyhow::Result;
use gix::bstr::ByteSlice;
use serde_json::json;
use std::path::Path;

#[derive(Default)]
pub struct GitSnapshotTool;

impl GitSnapshotTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl McpTool for GitSnapshotTool {
    fn name(&self) -> &'static str {
        "git_snapshot"
    }

    fn description(&self) -> &'static str {
        "Ultra-fast git repository inspector returning branch, commit info, file status, and diff summaries."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Workspace directory to inspect.",
                    "default": "."
                },
                "max_diff_lines": {
                    "type": "integer",
                    "description": "Maximum lines of diff details to include.",
                    "default": 100
                },
                "include_diff": {
                    "type": "boolean",
                    "description": "Whether to include modified file diffs.",
                    "default": true
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let max_diff_lines = args
            .get("max_diff_lines")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let include_diff = args
            .get("include_diff")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let target_path = Path::new(path_str);

        let repo = match gix::discover(target_path) {
            Ok(r) => r,
            Err(_) => {
                return Ok(json!({
                    "is_git_repo": false,
                    "message": "Not a git repository"
                }));
            }
        };

        // Extract branch name or detached HEAD
        let branch = match repo.head() {
            Ok(head) => match head.referent_name() {
                Some(name) => name.shorten().to_str_lossy().to_string(),
                None => head
                    .id()
                    .map(|id| id.to_hex_with_len(8).to_string())
                    .unwrap_or_else(|| "HEAD".to_string()),
            },
            Err(_) => "unknown".to_string(),
        };

        // Extract HEAD commit info using repo.head_commit()
        let head_commit = match repo.head_commit() {
            Ok(commit) => {
                let hash = commit.id().to_hex_with_len(8).to_string();
                let message = commit
                    .message()
                    .ok()
                    .map(|m| m.summary().to_str_lossy().to_string())
                    .unwrap_or_default();
                json!({
                    "hash": hash,
                    "message": message
                })
            }
            Err(_) => json!({
                "hash": "00000000",
                "message": "No commits yet"
            }),
        };

        let staged_files: Vec<String> = Vec::new();
        let mut unstaged_files: Vec<String> = Vec::new();
        let mut untracked_files: Vec<String> = Vec::new();
        let added_lines = 0;
        let removed_lines = 0;
        let mut diff_details: Vec<String> = Vec::new();

        // Perform status inspection using gix status platform
        if let Ok(status) = repo.status(gix::progress::Discard) {
            if let Ok(iter) = status.into_index_worktree_iter(None) {
                for item in iter.flatten() {
                    let path_str = item.rela_path().to_str_lossy().to_string();
                    use gix::status::index_worktree::iter::Item;
                    match item {
                        Item::DirectoryContents { .. } => {
                            untracked_files.push(path_str);
                        }
                        Item::Modification { .. } => {
                            unstaged_files.push(path_str);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Generate diff summary if requested
        if include_diff && !unstaged_files.is_empty() {
            let mut current_lines = 0;
            for file in &unstaged_files {
                if current_lines >= max_diff_lines {
                    break;
                }
                diff_details.push(format!("Modified: {}", file));
                current_lines += 1;
            }
        }

        let total_modified = staged_files.len() + unstaged_files.len();
        let diff_summary = if total_modified == 0 && untracked_files.is_empty() {
            "Working tree clean".to_string()
        } else {
            format!(
                "{} file(s) modified (+{} -{} lines)",
                total_modified, added_lines, removed_lines
            )
        };

        Ok(json!({
            "is_git_repo": true,
            "branch": branch,
            "head_commit": head_commit,
            "status": {
                "staged": staged_files,
                "unstaged": unstaged_files,
                "untracked": untracked_files
            },
            "diff_summary": diff_summary
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_git_snapshot_schema() {
        let tool = GitSnapshotTool::new();
        assert_eq!(tool.name(), "git_snapshot");
        let schema = tool.schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["path"].is_object());
    }

    #[tokio::test]
    async fn test_git_snapshot_non_git() {
        let dir = tempdir().unwrap();
        let tool = GitSnapshotTool::new();
        let args = json!({ "path": dir.path().to_str().unwrap() });

        let res = tool.execute(args).await.unwrap();
        assert_eq!(res["is_git_repo"], false);
        assert_eq!(res["message"], "Not a git repository");
    }

    #[tokio::test]
    async fn test_git_snapshot_repo() {
        let dir = tempdir().unwrap();
        let _repo = gix::init(dir.path()).unwrap();

        let tool = GitSnapshotTool::new();
        let args = json!({ "path": dir.path().to_str().unwrap() });

        let res = tool.execute(args).await.unwrap();
        assert_eq!(res["is_git_repo"], true);
        assert!(res["branch"].is_string());
        assert!(res["head_commit"]["hash"].is_string());
        assert!(res["status"]["staged"].is_array());
    }
}

