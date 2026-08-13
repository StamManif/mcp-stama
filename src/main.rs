use clap::Parser;
use mcp_stama::cli::Cli;
use mcp_stama::{StdioTransport, ToolRegistry, Transport};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI options
    let cli = Cli::parse();

    // CRITICAL: Configure tracing to write ALL logs strictly to stderr so stdout remains pure JSON-RPC
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let registry = Arc::new(ToolRegistry::new());
    registry.register(Arc::new(mcp_stama::tools::FastGrepTool::new()));
    registry.register(Arc::new(mcp_stama::tools::GitSnapshotTool::new()));
    registry.register(Arc::new(mcp_stama::tools::DockerWatcherTool::new()));

    if cli.benchmark {
        mcp_stama::benchmark::run_benchmark(registry).await?;
        return Ok(());
    }

    if cli.install_cursor || cli.install_claude {
        mcp_stama::installer::run_installer(cli.install_cursor, cli.install_claude)?;
        return Ok(());
    }

    info!("Starting mcp-stama server...");
    let mut transport = StdioTransport::new();
    info!("mcp-stama initialized. Ready for JSON-RPC messages via stdio.");

    while let Ok(maybe_req) = transport.read_request().await {
        match maybe_req {
            Some(req) => {
                let response = registry.handle_request(req).await;
                if let Err(e) = transport.send_response(&response).await {
                    error!("Failed to send response: {:?}", e);
                }
            }
            None => {
                info!("EOF or empty line received. Transport loop exiting.");
                break;
            }
        }
    }

    info!("mcp-stama server stopped.");
    Ok(())
}
