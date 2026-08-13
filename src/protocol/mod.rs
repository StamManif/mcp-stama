use serde::{Deserialize, Serialize};

/// Standard JSON-RPC 2.0 Request structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Standard JSON-RPC 2.0 Error structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, thiserror::Error)]
#[error("JSON-RPC Error ({code}): {message}")]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    pub fn new(code: i32, message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }

    pub fn parse_error(details: impl Into<String>) -> Self {
        Self::new(
            Self::PARSE_ERROR,
            "Parse error",
            Some(serde_json::json!({ "details": details.into() })),
        )
    }

    pub fn invalid_request(details: impl Into<String>) -> Self {
        Self::new(
            Self::INVALID_REQUEST,
            "Invalid Request",
            Some(serde_json::json!({ "details": details.into() })),
        )
    }

    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::new(
            Self::METHOD_NOT_FOUND,
            format!("Method not found: {}", method.into()),
            None,
        )
    }

    pub fn invalid_params(details: impl Into<String>) -> Self {
        Self::new(
            Self::INVALID_PARAMS,
            "Invalid params",
            Some(serde_json::json!({ "details": details.into() })),
        )
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(Self::INTERNAL_ERROR, message, None)
    }
}

/// Standard JSON-RPC 2.0 Response structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };
        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn test_response_success() {
        let resp = JsonRpcResponse::success(
            Some(serde_json::json!("abc")),
            serde_json::json!({ "status": "ok" }),
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap(), serde_json::json!({ "status": "ok" }));
    }

    #[test]
    fn test_error_construction() {
        let err = JsonRpcError::method_not_found("test_method");
        assert_eq!(err.code, JsonRpcError::METHOD_NOT_FOUND);
        assert_eq!(err.message, "Method not found: test_method");
    }
}

