use clap::Parser;

/// CLI argument definitions for mcp-stama
#[derive(Parser, Debug, Clone)]
#[command(
    name = "mcp-stama",
    version,
    about = "High-performance, zero-dependency Model Context Protocol (MCP) server written in Rust"
)]
pub struct Cli {
    /// Run micro-latency benchmark suite across all registered tools
    #[arg(short, long)]
    pub benchmark: bool,

    /// Install mcp-stama into local Cursor configuration
    #[arg(long)]
    pub install_cursor: bool,

    /// Install mcp-stama into Claude Desktop configuration
    #[arg(long)]
    pub install_claude: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_default() {
        let args = vec!["mcp-stama"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(!cli.benchmark);
        assert!(!cli.install_cursor);
        assert!(!cli.install_claude);
    }

    #[test]
    fn test_cli_parsing_benchmark() {
        let args = vec!["mcp-stama", "--benchmark"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.benchmark);

        let args_short = vec!["mcp-stama", "-b"];
        let cli_short = Cli::try_parse_from(args_short).unwrap();
        assert!(cli_short.benchmark);
    }

    #[test]
    fn test_cli_parsing_installers() {
        let args = vec!["mcp-stama", "--install-cursor", "--install-claude"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.install_cursor);
        assert!(cli.install_claude);
    }
}
