// ---------------------------------------------------------------------------
//  Thunder Blockchain — JSON-RPC API Server
// ---------------------------------------------------------------------------
//  Provides a JSON-RPC 2.0 interface for external applications to interact
//  with the blockchain.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

// ── JSON-RPC Types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: u64, code: i64, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
            }),
            id,
        }
    }
}

// ── RPC Handler ────────────────────────────────────────────────────────────

/// Handles JSON-RPC method dispatch.
pub struct RpcHandler;

impl RpcHandler {
    /// Route a JSON-RPC request to the appropriate handler.
    pub fn handle(request: &JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "thunder_chainId" => JsonRpcResponse::success(
                request.id,
                serde_json::json!({ "chain_id": "thunder-mainnet-1" }),
            ),

            "thunder_blockNumber" => {
                // In production, this would query the node's chain.
                JsonRpcResponse::success(request.id, serde_json::json!({ "height": 0 }))
            }

            "thunder_getBalance" => {
                let address = request
                    .params
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0x0");
                // Placeholder — in production, look up from WorldState.
                JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({ "address": address, "balance": "0" }),
                )
            }

            "thunder_sendTransaction" => {
                // Placeholder — in production, validate, sign-check, and add to mempool.
                JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({ "tx_hash": "0x0000000000000000" }),
                )
            }

            "thunder_getBlock" => {
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({
                        "height": height,
                        "hash": "0x0000000000000000",
                        "transactions": []
                    }),
                )
            }

            "thunder_getValidators" => {
                JsonRpcResponse::success(request.id, serde_json::json!({ "validators": [] }))
            }

            "thunder_compileContract" => {
                let source = request
                    .params
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                match thunder_lang::compile_source(source) {
                    Ok(compiled) => JsonRpcResponse::success(
                        request.id,
                        serde_json::json!({
                            "name": compiled.name,
                            "bytecode_length": compiled.instructions.len(),
                            "functions": compiled.function_table.keys()
                                .collect::<Vec<_>>(),
                            "state_slots": compiled.state_slots,
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(request.id, -32000, &e),
                }
            }

            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                &format!("method '{}' not found", request.method),
            ),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(method: &str, params: serde_json::Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        }
    }

    #[test]
    fn test_chain_id() {
        let req = make_request("thunder_chainId", serde_json::json!({}));
        let res = RpcHandler::handle(&req);
        assert!(res.result.is_some());
        assert!(res.error.is_none());
    }

    #[test]
    fn test_unknown_method() {
        let req = make_request("unknown_method", serde_json::json!({}));
        let res = RpcHandler::handle(&req);
        assert!(res.error.is_some());
        assert_eq!(res.error.unwrap().code, -32601);
    }

    #[test]
    fn test_compile_contract_via_rpc() {
        let src = r#"contract Test { fn hello() -> u64 { return 42; } }"#;
        let req = make_request(
            "thunder_compileContract",
            serde_json::json!({ "source": src }),
        );
        let res = RpcHandler::handle(&req);
        assert!(res.result.is_some());
        let result = res.result.unwrap();
        assert_eq!(result["name"], "Test");
    }

    #[test]
    fn test_compile_invalid_contract() {
        let req = make_request(
            "thunder_compileContract",
            serde_json::json!({ "source": "invalid code @@@@" }),
        );
        let res = RpcHandler::handle(&req);
        assert!(res.error.is_some());
    }
}
