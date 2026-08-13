use crate::engine::ToolRegistry;
use crate::protocol::JsonRpcRequest;
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{MemoryRefreshKind, Pid, RefreshKind, System};

pub async fn run_benchmark(registry: Arc<ToolRegistry>) -> anyhow::Result<()> {
    eprintln!("\nRunning mcp-stama Micro-Latency Benchmark Suite\n");

    let tools_list = registry.list_tools();
    if tools_list.is_empty() {
        eprintln!("No registered tools found to benchmark.");
        return Ok(());
    }

    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );

    sys.refresh_memory();
    let pid = Pid::from_u32(std::process::id());
    let memory_bytes = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
    let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);

    eprintln!(
        "{:<18} | {:<12} | {:<12} | {:<12} | {:<14}",
        "Tool Name", "Invocations", "p50 Latency", "p99 Latency", "Memory RSS"
    );
    eprintln!("{:-<72}", "");

    for tool_info in &tools_list {
        let name = tool_info
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        let sample_req = match name {
            "fast_grep" => JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(serde_json::json!(1)),
                method: "tools/call".to_string(),
                params: Some(serde_json::json!({
                    "name": "fast_grep",
                    "arguments": { "query": "fn main", "path": "src" }
                })),
            },

            "git_snapshot" => JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(serde_json::json!(1)),
                method: "tools/call".to_string(),
                params: Some(serde_json::json!({
                    "name": "git_snapshot",
                    "arguments": { "path": "." }
                })),
            },
            "docker_watcher" => JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(serde_json::json!(1)),
                method: "tools/call".to_string(),
                params: Some(serde_json::json!({
                    "name": "docker_watcher",
                    "arguments": {}
                })),
            },
            _ => JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(serde_json::json!(1)),
                method: name.to_string(),
                params: None,
            },
        };

        // Warm-up (10 iterations)
        for _ in 0..10 {
            let _ = registry.handle_request(sample_req.clone()).await;
        }

        // Benchmark loop (100 iterations)
        let total_invocations = 100;
        let mut latencies_micros = Vec::with_capacity(total_invocations);

        for _ in 0..total_invocations {
            let start = Instant::now();
            let _ = registry.handle_request(sample_req.clone()).await;
            latencies_micros.push(start.elapsed().as_micros() as u64);
        }


        latencies_micros.sort_unstable();

        let p50_micros = latencies_micros[total_invocations / 2];
        let p99_micros = latencies_micros[(total_invocations * 99) / 100];

        eprintln!(
            "{:<18} | {:<12} | {:<12} | {:<12} | {:.2} MB",
            name,
            total_invocations,
            format_duration(p50_micros),
            format_duration(p99_micros),
            memory_mb
        );
    }

    eprintln!("\nBenchmark completed successfully.\n");
    Ok(())
}

fn format_duration(micros: u64) -> String {
    if micros < 1000 {
        format!("{} µs", micros)
    } else {
        format!("{:.2} ms", micros as f64 / 1000.0)
    }
}
