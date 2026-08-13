#[async_trait::async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

pub mod fast_grep;
pub use fast_grep::FastGrepTool;

pub mod git_snapshot;
pub use git_snapshot::GitSnapshotTool;

pub mod docker_watcher;
pub use docker_watcher::DockerWatcherTool;



