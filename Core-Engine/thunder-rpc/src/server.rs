// ---------------------------------------------------------------------------
//  Thunder Blockchain — JSON-RPC API Server (Infura Testnet Architecture)
// ---------------------------------------------------------------------------
//  Provides a JSON-RPC 2.0 interface for external applications.
//  Now wired with RpcContext for integrating WorldState & Mempool.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

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

// ── RPC Context (Infura Foundation) ────────────────────────────────────────

/// Shared state context injected into the RPC Server.
pub struct RpcContext {
    pub chain_id: u64,
    pub genesis_addr: String,
    pub node: std::sync::Arc<std::sync::RwLock<thunder_network::node::Node>>,
}

impl RpcContext {
    pub fn new(
        chain_id: u64,
        genesis_addr: String,
        node: std::sync::Arc<std::sync::RwLock<thunder_network::node::Node>>,
    ) -> Self {
        Self {
            chain_id,
            genesis_addr,
            node,
        }
    }
}

// ── RPC Handler ────────────────────────────────────────────────────────────

/// Handles JSON-RPC method dispatch.
pub struct RpcHandler;

impl RpcHandler {
    /// Route a JSON-RPC request to the appropriate handler asynchronously.
    pub async fn handle(request: &JsonRpcRequest, ctx: Arc<RwLock<RpcContext>>) -> JsonRpcResponse {
        let context = ctx.read().await;

        match request.method.as_str() {
            "thunder_chainId" => JsonRpcResponse::success(
                request.id,
                serde_json::json!({ "chain_id": context.chain_id }),
            ),

            "thunder_blockNumber" => {
                let height = context.node.read().unwrap().height();
                JsonRpcResponse::success(request.id, serde_json::json!({ "height": height }))
            }

            "thunder_getBalance" => {
                let address_str = request
                    .params
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0x0");

                let mut balance = 0;
                if let Ok(addr) = thunder_core::crypto::address_from_hex(address_str) {
                    balance = context
                        .node
                        .read()
                        .unwrap()
                        .state
                        .read()
                        .unwrap()
                        .get_balance(&addr);
                }

                JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({ "address": address_str, "balance": balance.to_string() }),
                )
            }

            "thunder_gasPrice" => {
                let node = context.node.read().unwrap();
                // Dynamic EIP-1559 logic: Base 1 Gwei + 5 Gwei penalty per pending mempool transaction (Congestion/Difficulty)
                let congestion_penalty = (node.mempool.len() as u64) * 5;
                let dynamic_gwei = 1 + congestion_penalty;
                JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({ "gas_price": dynamic_gwei }),
                )
            }

            "thunder_sendTransaction" => {
                let data = request
                    .params
                    .get("data")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let parse_res = hex::decode(data)
                    .map_err(|_| "Invalid Hex")
                    .and_then(|bytes| {
                        bincode::deserialize::<thunder_core::transaction::Transaction>(&bytes)
                            .map_err(|_| "Deserialization failed")
                    });

                match parse_res {
                    Ok(tx) => {
                        if tx.chain_id != context.chain_id {
                            return JsonRpcResponse::error(
                                request.id,
                                -32000,
                                &format!("Invalid Chain ID. Expected {}, got {}", context.chain_id, tx.chain_id),
                            );
                        }
                        
                        let hash = thunder_core::crypto::hash_to_hex(&tx.hash());
                        match context.node.write().unwrap().add_transaction(tx) {
                            Ok(_) => JsonRpcResponse::success(
                                request.id,
                                serde_json::json!({ "tx_hash": format!("0x{}", hash) }),
                            ),
                            Err(e) => JsonRpcResponse::error(request.id, -32001, &e),
                        }
                    }
                    Err(e) => JsonRpcResponse::error(request.id, -32602, e),
                }
            }

            "thunder_registerValidator" => {
                let address_hex = request
                    .params
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let pubkey_hex = request
                    .params
                    .get("public_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let stake = request
                    .params
                    .get("stake")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                match (
                    thunder_core::crypto::address_from_hex(address_hex),
                    hex::decode(pubkey_hex),
                ) {
                    (Ok(addr), Ok(pk_bytes)) if pk_bytes.len() == 32 => {
                        let mut pk = [0u8; 32];
                        pk.copy_from_slice(&pk_bytes);

                        let mut n = context.node.write().unwrap();

                        if let Err(e) = n.validator_set.register(addr, pk, stake) {
                            return JsonRpcResponse::error(request.id, -32000, &e.to_string());
                        }

                        n.consensus.validators = n
                            .validator_set
                            .active_validators()
                            .iter()
                            .map(|v| v.address)
                            .collect();

                        JsonRpcResponse::success(
                            request.id,
                            serde_json::json!("Validator Registered Successfully"),
                        )
                    }
                    _ => JsonRpcResponse::error(request.id, -32602, "Invalid address or pubkey"),
                }
            }

            "thunder_requestFaucet" => {
                let recipient_hex = request
                    .params
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let requested_amount = request
                    .params
                    .get("amount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100000);
                let dynamic_gas = request
                    .params
                    .get("gas_price")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1);
                match thunder_core::crypto::address_from_hex(recipient_hex) {
                    Ok(to_addr) => {
                        let mut n = context.node.write().unwrap();
                        let unique_nonce = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .subsec_nanos() as u64;
                        let mut tx = thunder_core::transaction::Transaction::new_transfer(1, 
                            unique_nonce,
                            n.key_pair.address(),
                            to_addr,
                            requested_amount,
                            21000,
                            dynamic_gas,
                        );
                        // Node securely signs testnet funds with the master Genesis KeyPair
                        tx.sign(&n.key_pair);
                        let hash = thunder_core::crypto::hash_to_hex(&tx.hash());
                        match n.add_transaction(tx) {
                            Ok(_) => JsonRpcResponse::success(
                                request.id,
                                serde_json::json!({ "tx_hash": format!("0x{}", hash), "amount": requested_amount }),
                            ),
                            Err(e) => JsonRpcResponse::error(request.id, -32001, &e),
                        }
                    }
                    Err(_) => JsonRpcResponse::error(request.id, -32602, "Invalid Address Format"),
                }
            }

            "thunder_getMempool" => {
                let node = context.node.read().unwrap();
                let mempool = &node.mempool;

                let mapped_pool: Vec<serde_json::Value> = mempool
                    .iter()
                    .map(|tx| {
                        serde_json::json!({
                            "hash": format!("0x{}", thunder_core::crypto::hash_to_hex(&tx.hash())),
                            "from": thunder_core::crypto::address_to_hex(&tx.from),
                            "to": thunder_core::crypto::address_to_hex(&tx.to),
                            "value": tx.value,
                            "gas_limit": tx.gas_limit,
                            "gas_price": tx.gas_price,
                            "kind": format!("{:?}", tx.kind)
                        })
                    })
                    .collect();

                JsonRpcResponse::success(request.id, serde_json::json!({ "mempool": mapped_pool }))
            }

            "thunder_getBlock" => {
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let node = context.node.read().unwrap();
                if let Some(block) = node.get_block(height) {
                    let mut gas_used = 0u64;
                    let mut fees_nanothdr = 0u64;

                    let mapped_txs: Vec<serde_json::Value> = block.transactions.iter().map(|tx| {
                        gas_used += tx.gas_limit;
                        fees_nanothdr += tx.gas_limit * tx.gas_price;

                        serde_json::json!({
                            "hash": format!("0x{}", thunder_core::crypto::hash_to_hex(&tx.hash())),
                            "from": thunder_core::crypto::address_to_hex(&tx.from),
                            "to": thunder_core::crypto::address_to_hex(&tx.to),
                            "value": tx.value,
                            "gas_limit": tx.gas_limit,
                            "gas_price": tx.gas_price,
                            "kind": format!("{:?}", tx.kind),
                            "time": 0,
                            "timestamp": block.header.timestamp
                        })
                    }).collect();

                    let block_size = 1284 + (block.transactions.len() * 256);
                    let final_reward = block.header.reward as f64 / 1_000_000_000.0;

                    JsonRpcResponse::success(
                        request.id,
                        serde_json::json!({
                            "height": block.header.height,
                            "hash": format!("0x{}", thunder_core::crypto::hash_to_hex(&block.hash())),
                            "transactions": mapped_txs,
                            "timestamp": block.header.timestamp,
                            "validator": thunder_core::crypto::address_to_hex(&block.header.validator),
                            "txn_count": block.transactions.len(),
                            "gas_used": gas_used,
                            "gas_limit": 30_000_000,
                            "base_fee": block.header.base_fee,
                            "reward": final_reward,
                            "size": block_size
                        }),
                    )
                } else {
                    JsonRpcResponse::error(request.id, -32602, "block not found")
                }
            }

            "thunder_getTransactionByHash" => {
                let tx_hash_str = request
                    .params
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let node = context.node.read().unwrap();
                let mut found_tx = None;

                // Check Mempool First
                for tx in &node.mempool {
                    if format!("0x{}", thunder_core::crypto::hash_to_hex(&tx.hash())) == tx_hash_str
                    {
                        found_tx = Some((tx.clone(), None));
                        break;
                    }
                }

                // Check Historical Blocks
                if found_tx.is_none() {
                    for block in node.chain.iter().rev() {
                        if let Some(tx) = block.transactions.iter().find(|t| {
                            format!("0x{}", thunder_core::crypto::hash_to_hex(&t.hash()))
                                == tx_hash_str
                        }) {
                            found_tx = Some((
                                tx.clone(),
                                Some((block.header.height, block.header.timestamp)),
                            ));
                            break;
                        }
                    }
                }

                if let Some((tx, height_opt)) = found_tx {
                    let mut payload = serde_json::json!({
                        "hash": format!("0x{}", thunder_core::crypto::hash_to_hex(&tx.hash())),
                        "from": thunder_core::crypto::address_to_hex(&tx.from),
                        "to": thunder_core::crypto::address_to_hex(&tx.to),
                        "value": tx.value,
                        "gas_limit": tx.gas_limit,
                        "gas_price": tx.gas_price,
                        "kind": format!("{:?}", tx.kind),
                        "time": 0
                    });

                    if let Some((h, ts)) = height_opt {
                        payload
                            .as_object_mut()
                            .unwrap()
                            .insert("block_height".to_string(), serde_json::json!(h));
                        payload
                            .as_object_mut()
                            .unwrap()
                            .insert("timestamp".to_string(), serde_json::json!(ts));
                    } else {
                        payload
                            .as_object_mut()
                            .unwrap()
                            .insert("block_height".to_string(), serde_json::json!(null));
                        payload
                            .as_object_mut()
                            .unwrap()
                            .insert("timestamp".to_string(), serde_json::json!(null));
                    }

                    JsonRpcResponse::success(request.id, payload)
                } else {
                    JsonRpcResponse::error(request.id, -32602, "transaction not found")
                }
            }

            // ── Account Specific History ──
            "thunder_getTransactionsByAddress" => {
                let address_str = request
                    .params
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0x0");

                let node = context.node.read().unwrap();
                let mut txs = Vec::new();

                for (i, block) in node.chain.iter().rev().enumerate() {
                    if i > 100 {
                        break;
                    } // Limit deep historical scans temporarily
                    for tx in block.transactions.iter().rev() {
                        let from_hex = thunder_core::crypto::address_to_hex(&tx.from);
                        let to_hex = thunder_core::crypto::address_to_hex(&tx.to);

                        if from_hex == address_str || to_hex == address_str {
                            txs.push(serde_json::json!({
                                "hash": format!("0x{}", thunder_core::crypto::hash_to_hex(&tx.hash())),
                                "from": from_hex,
                                "to": to_hex,
                                "value": tx.value,
                                "gas_limit": tx.gas_limit,
                                "gas_price": tx.gas_price,
                                "kind": format!("{:?}", tx.kind),
                                "time": 0,
                                "timestamp": block.header.timestamp,
                                "block_height": block.header.height
                            }));
                        }
                    }
                }
                JsonRpcResponse::success(request.id, serde_json::json!({ "transactions": txs }))
            }

            "thunder_getValidators" => {
                let node = context.node.read().unwrap();
                let mut vals = Vec::new();
                for val in node.validator_set.active_validators() {
                    vals.push(serde_json::json!({
                        "address": thunder_core::crypto::address_to_hex(&val.address),
                        "name": "ThunderNode (Testnet)",
                        "public_key": format!("0x{}", thunder_core::crypto::hash_to_hex(&[0; 32])), // Fallback dummy public key print
                        "stake": val.stake,
                        "is_active": true
                    }));
                }

                JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({
                        "validators": vals
                    }),
                )
            }

            "thunder_compileContract" => {
                let source = request
                    .params
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                match thunder_lang::compile_source(source) {
                    Ok(compiled) => {
                        let bytecode = hex::encode(bincode::serialize(&compiled).unwrap());
                        JsonRpcResponse::success(
                            request.id,
                            serde_json::json!({
                                "name": compiled.name,
                                "bytecode_length": compiled.instructions.len(),
                                "functions": compiled.function_table.keys()
                                    .collect::<Vec<_>>(),
                                "state_slots": compiled.state_slots,
                                "bytecode": bytecode,
                            }),
                        )
                    },
                    Err(e) => JsonRpcResponse::error(request.id, -32000, &e),
                }
            }

            "thunder_call" => {
                let to_hex = request
                    .params
                    .get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let data = request
                    .params
                    .get("data")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                match (thunder_core::crypto::address_from_hex(to_hex), hex::decode(data)) {
                    (Ok(to_addr), Ok(_call_data)) => {
                        let node = context.node.read().unwrap();
                        let state = node.state.read().unwrap();

                        let account = state.get_account(&to_addr);
                        if !account.code.is_empty() {
                            let bytecode = &account.code;
                            match bincode::deserialize::<thunder_lang::compiler::CompiledContract>(bytecode) {
                                Ok(compiled) => {
                                    let ctx = thunder_vm::vm::ExecutionContext {
                                        caller: [0u8; 20],
                                        contract_address: to_addr,
                                        value: 0,
                                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                                        block_height: node.height(),
                                    };
                                        
                                        // A real system would map call_data to arguments and set the PC to the requested function
                                        // For now, we just run the bytecode from start
                                        let mut vm = thunder_vm::vm::ThunderVm::new(
                                            compiled.instructions,
                                            ctx,
                                            10_000_000,
                                            1,
                                            account.storage.clone()
                                        );
                                        
                                        match vm.execute() {
                                            Ok(res) => JsonRpcResponse::success(
                                                request.id,
                                                serde_json::json!({
                                                    "return_value": res.return_value,
                                                    "reverted": res.reverted,
                                                    "revert_reason": res.revert_reason,
                                                })
                                            ),
                                            Err(e) => JsonRpcResponse::error(request.id, -32000, &format!("VM Error: {:?}", e)),
                                        }
                                    },
                                    Err(_) => JsonRpcResponse::error(request.id, -32000, "Failed to deserialize contract code"),
                                }
                            } else {
                                JsonRpcResponse::error(request.id, -32000, "Address is not a contract")
                            }
                    },
                    _ => JsonRpcResponse::error(request.id, -32602, "Invalid arguments"),
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

    async fn get_ctx() -> Arc<RwLock<RpcContext>> {
        use thunder_core::crypto::KeyPair;
        use thunder_network::node::NodeConfig;
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);

        let config = NodeConfig {
            data_dir: format!(
                "/tmp/thunder_test_node_rpc_{}_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                id
            ),
            listen_port: 3000,
            max_peers: 5,
            min_stake: 1000,
        };
        let kp = KeyPair::generate();
        let node = std::sync::Arc::new(std::sync::RwLock::new(thunder_network::node::Node::new(
            kp, config,
        )));
        Arc::new(RwLock::new(RpcContext::new(
            1,
            "0x0".to_string(),
            node,
        )))
    }

    #[tokio::test]
    async fn test_chain_id() {
        let req = make_request("thunder_chainId", serde_json::json!({}));
        let res = RpcHandler::handle(&req, get_ctx().await).await;
        assert!(res.result.is_some());
        assert_eq!(res.result.unwrap()["chain_id"], 1);
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let req = make_request("unknown_method", serde_json::json!({}));
        let res = RpcHandler::handle(&req, get_ctx().await).await;
        assert!(res.error.is_some());
        assert_eq!(res.error.unwrap().code, -32601);
    }
}

// ── HTTP API Server (Warp) ─────────────────────────────────────────────────

use std::net::SocketAddr;
use warp::Filter;

/// Helper to inject context into warp requests
fn with_context(
    ctx: Arc<RwLock<RpcContext>>,
) -> impl Filter<Extract = (Arc<RwLock<RpcContext>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}

/// Start the JSON-RPC API Server asynchronously (The Infura node).
/// Start the JSON-RPC API Server asynchronously (The Infura node).
pub async fn start_server(
    port: u16,
    chain_id: u64,
    genesis_addr: String,
    shared_node: Arc<std::sync::RwLock<thunder_network::node::Node>>,
) {
    let context = Arc::new(RwLock::new(RpcContext::new(
        chain_id,
        genesis_addr,
        shared_node,
    )));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST", "OPTIONS"])
        .allow_headers(vec!["content-type"]);

    let route = warp::post()
        .and(warp::body::json())
        .and(with_context(context))
        .then(
            |req: JsonRpcRequest, ctx: Arc<RwLock<RpcContext>>| async move {
                let res = RpcHandler::handle(&req, ctx).await;
                warp::reply::json(&res)
            },
        )
        .with(cors);

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    tracing::info!(
        "Starting Thunder JSON-RPC Testnet Node on http://{} ⚡",
        addr
    );
    warp::serve(route).run(addr).await;
}
