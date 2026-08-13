use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::tools::McpTool;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Thread-safe registry for storing and managing registered MCP tools.
#[derive(Default)]
pub struct ToolRegistry {
    tools: DashMap<String, Arc<dyn McpTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: DashMap::new(),
        }
    }

    /// Register a new tool in the registry.
    pub fn register(&self, tool: Arc<dyn McpTool>) {
        info!("Registering tool: {}", tool.name());
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Retrieve a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn McpTool>> {
        self.tools.get(name).map(|r| Arc::clone(r.value()))
    }

    /// Returns definitions of all registered tools.
    pub fn list_tools(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .map(|r| {
                let tool = r.value();
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "inputSchema": tool.schema(),
                })
            })
            .collect()
    }

    /// Handle an incoming JSON-RPC request and dispatch to tool or handle MCP protocol calls.
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let req_id = req.id.clone();
        match req.method.as_str() {
            "initialize" => {
                let result = serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "mcp-turbo",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                JsonRpcResponse::success(req_id, result)
            }
            "ping" => JsonRpcResponse::success(req_id, serde_json::json!({})),
            "tools/list" => {
                let tools_list = self.list_tools();
                JsonRpcResponse::success(req_id, serde_json::json!({ "tools": tools_list }))
            }
            "tools/call" => {
                let params = match req.params {
                    Some(p) => p,
                    None => {
                        return JsonRpcResponse::error(
                            req_id,
                            JsonRpcError::invalid_params("Missing params for tools/call"),
                        );
                    }
                };

                let name = match params.get("name").and_then(|n| n.as_str()) {
                    Some(n) => n,
                    None => {
                        return JsonRpcResponse::error(
                            req_id,
                            JsonRpcError::invalid_params("Missing 'name' field in tools/call params"),
                        );
                    }
                };

                let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

                let tool = match self.get(name) {
                    Some(t) => t,
                    None => {
                        warn!("Tool not found: {}", name);
                        return JsonRpcResponse::error(
                            req_id,
                            JsonRpcError::method_not_found(format!("Tool '{}' not found", name)),
                        );
                    }
                };

                match tool.execute(arguments).await {
                    Ok(val) => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string())
                                }
                            ]
                        }),
                    ),
                    Err(e) => {
                        warn!("Execution error in tool '{}': {:?}", name, e);
                        JsonRpcResponse::error(
                            req_id,
                            JsonRpcError::internal_error(format!("Tool execution failed: {}", e)),
                        )
                    }
                }
            }

            // Fallback for direct tool call method name
            method_name => {
                if let Some(tool) = self.get(method_name) {
                    let args = req.params.unwrap_or(serde_json::json!({}));
                    match tool.execute(args).await {
                        Ok(res) => JsonRpcResponse::success(req_id, res),
                        Err(e) => JsonRpcResponse::error(
                            req_id,
                            JsonRpcError::internal_error(format!("Tool execution failed: {}", e)),
                        ),
                    }
                } else {
                    JsonRpcResponse::error(req_id, JsonRpcError::method_not_found(method_name))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyTool;

    #[async_trait::async_trait]
    impl McpTool for DummyTool {
        fn name(&self) -> &'static str {
            "dummy_tool"
        }

        fn description(&self) -> &'static str {
            "A dummy test tool"
        }

        fn schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            })
        }

        async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("default");
            Ok(serde_json::json!({ "result": format!("echo: {}", input) }))
        }
    }

    #[tokio::test]
    async fn test_registry_registration_and_list() {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool));

        assert!(registry.get("dummy_tool").is_some());
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "dummy_tool");
    }

    #[tokio::test]
    async fn test_handle_request_tools_list() {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool));

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };

        let resp = registry.handle_request(req).await;
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.error.is_none());
        assert!(resp.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_request_tools_call() {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool));

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(2)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "dummy_tool",
                "arguments": { "input": "hello" }
            })),
        };

        let resp = registry.handle_request(req).await;
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert!(result.get("content").is_some());
    }
}

