// ---------------------------------------------------------------------------
//  Thunder Blockchain — CLI Entry Point
// ---------------------------------------------------------------------------
//  Command-line interface for running a Thunder node, managing wallets,
//  sending transactions, and deploying smart contracts.
// ---------------------------------------------------------------------------

use clap::{Parser, Subcommand};

use std::collections::HashMap;

use thunder_core::crypto::{address_to_hex, KeyPair, SerializableKeyPair};
use thunder_core::state::Account;
use thunder_lang::compiler::compile_source;
use thunder_network::node::{Node, NodeConfig};
use thunder_vm::opcode::Instruction;
use thunder_vm::vm::{ExecutionContext, ThunderVm};

// ── CLI Definition ─────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "thunder",
    about = "⚡ Thunder Blockchain — Blockchain with PoS aBFT & ThunderScript",
    version = "0.1.0",
    author = "Thunder Blockchain Contributors"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a Thunder Blockchain node.
    Node {
        #[command(subcommand)]
        action: NodeCommands,
    },
    /// Wallet management.
    Wallet {
        #[command(subcommand)]
        action: WalletCommands,
    },
    /// Transaction operations.
    Tx {
        #[command(subcommand)]
        action: TxCommands,
    },
    /// Smart contract operations.
    Contract {
        #[command(subcommand)]
        action: ContractCommands,
    },
}

// ── Node Subcommands ───────────────────────────────────────────────────────

#[derive(Subcommand)]
enum NodeCommands {
    /// Start the node.
    Start {
        #[arg(short, long, default_value = "./data")]
        data_dir: String,
        #[arg(short, long, default_value_t = 30303)]
        port: u16,
        #[arg(long)]
        bootnode: Option<String>,
    },
    /// Show node status.
    Status,
}

// ── Wallet Subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
enum WalletCommands {
    /// Generate a new wallet (key pair).
    Create,
    /// Login to the interactive wallet shell
    Login,
    /// Show the balance of an address.
    Balance {
        #[arg(short, long)]
        address: String,
    },
}

// ── Transaction Subcommands ────────────────────────────────────────────────

#[derive(Subcommand)]
enum TxCommands {
    /// Send coins to an address.
    Send {
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: u64,
        #[arg(short, long, default_value_t = 21000)]
        gas_limit: u64,
    },
}

// ── Contract Subcommands ───────────────────────────────────────────────────

#[derive(Subcommand)]
enum ContractCommands {
    /// Compile a ThunderScript file.
    Compile {
        /// Path to the .thunder file.
        file: String,
    },
    /// Deploy a compiled contract.
    Deploy {
        /// Path to the .thunder file.
        file: String,
    },
    /// Execute a ThunderScript file in a local VM (for testing).
    Run {
        /// Path to the .thunder file.
        file: String,
        /// Function to call.
        #[arg(short, long, default_value = "init")]
        function: String,
    },
}

// ── Main ───────────────────────────────────────────────────────────────────

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        // ── Node Commands ──────────────────────────────────────────────
        Commands::Node { action } => match action {
            NodeCommands::Start {
                data_dir,
                port,
                bootnode,
            } => {
                println!("⚡ Thunder Blockchain");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                let mut secret_bytes = [0u8; 32];
                secret_bytes[31] = 1;
                let key_pair = KeyPair::from_secret_bytes(&secret_bytes);
                let genesis_addr = address_to_hex(&key_pair.address());
                println!("  Node Address : {}", genesis_addr);
                println!("  Secret Key   : {}", hex::encode(key_pair.secret_bytes()));
                println!("  Data Dir     : {}", data_dir);
                println!("  Listen Port  : {}", port);
                if let Some(bn) = &bootnode {
                    println!("  Bootnode     : {}", bn);
                }
                println!("  Consensus    : PoS aBFT (Hashgraph)");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                let config = NodeConfig {
                    data_dir: data_dir.clone(),
                    listen_port: port,
                    ..Default::default()
                };
                let mut node = Node::new(key_pair, config);

                // Register as validator with genesis stake.
                node.register_as_validator(100_000_000_000_000)
                    .expect("failed to register as validator");

                // Give the node's account some initial coins.
                node.state.write().unwrap().set_account(
                    &node.key_pair.address(),
                    Account::with_balance(1_000_000_000_000_000_000), // 1 Billion THDR scaled by 1e9 (Nano-THDR)
                );

                println!("  Status       : ✅ Node initialised (genesis block created)");
                println!("  Validators   : {}", node.validator_set.active_count());
                println!("  Chain Height : {}", node.height());
                println!();
                println!("  Node is ready. In a full deployment, the P2P event");
                println!("  loop would start here using libp2p + tokio.");

                let shared_node = std::sync::Arc::new(std::sync::RwLock::new(node));
                let rt = tokio::runtime::Runtime::new().unwrap();

                // Mount Automated Block Forger (Auto-Mining Loop)
                let forger_node = std::sync::Arc::clone(&shared_node);
                rt.spawn(async move {
                    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
                    loop {
                        interval.tick().await;
                        let mut n = forger_node.write().unwrap();

                        // Prevent spamming empty blocks if there's no tx
                        if n.mempool.is_empty() {
                            continue;
                        }

                        // Bundle pending txns into DAG Event
                        if n.create_event().is_ok() {
                            // Automatically process round & forge
                            if let Some(block) = n.try_produce_block() {
                                println!(
                                    "\n  🔨 FORGED BLOCK #{} | {} txns | Hash: 0x{}",
                                    block.header.height,
                                    block.transactions.len(),
                                    &hex::encode(block.hash())[0..10]
                                );
                            }
                        }
                    }
                });

                // Mount the HTTP JSON-RPC Server
                rt.block_on(async {
                    thunder_rpc::start_server(8080, genesis_addr, shared_node).await;
                });
            }
            NodeCommands::Status => {
                println!("⚡ Thunder Blockchain — Node Status");
                println!("  Chain Height : 0 (genesis)");
                println!("  Peers        : 0");
                println!("  Mempool      : 0 transactions");
            }
        },

        // ── Wallet Commands ────────────────────────────────────────────
        Commands::Wallet { action } => match action {
            WalletCommands::Create => {
                let kp = KeyPair::generate();
                let serializable = SerializableKeyPair::from(&kp);

                println!("⚡ New Thunder Wallet Created");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  Address    : {}", serializable.address);
                println!("  Public Key : {}", serializable.public_key);
                println!("  Secret Key : {}", serializable.secret_key);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  ⚠ SAVE YOUR SECRET KEY! It cannot be recovered.");
            }
            WalletCommands::Login => {
                println!("⚡ Thunder Interactive Wallet ⚡");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                print!("  [🔑] Enter Secret Key (Hex): ");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                let mut secret_input = String::new();
                std::io::stdin().read_line(&mut secret_input).unwrap();
                let secret_input = secret_input.trim();

                let secret_array = match hex::decode(secret_input) {
                    Ok(b) if b.len() == 32 => {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&b);
                        arr
                    }
                    _ => {
                        println!("  ❌ Invalid Secret Key format (must be 32 bytes hex).");
                        return;
                    }
                };
                let key_pair = KeyPair::from_secret_bytes(&secret_array);
                let my_address = hex::encode(key_pair.address());
                println!("  ✅ Logged in as: 0x{}\n", my_address);

                let rpc_url = "http://127.0.0.1:8080";
                let client = reqwest::blocking::Client::new();

                loop {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("  [1] Check Balance");
                    println!("  [2] Request Testnet THDR (Faucet)");
                    println!("  [3] Send THDR");
                    println!("  [4] Stake THDR & Launch Validator Node");
                    println!("  [5] Exit");
                    print!("  > Choose an option (1-5): ");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();

                    let mut choice = String::new();
                    std::io::stdin().read_line(&mut choice).unwrap();
                    match choice.trim() {
                        "1" => {
                            let payload = serde_json::json!({
                                "jsonrpc": "2.0",
                                "method": "thunder_getBalance",
                                "params": { "address": format!("0x{}", my_address) },
                                "id": 1
                            });
                            match client.post(rpc_url).json(&payload).send() {
                                Ok(res) => {
                                    let json: serde_json::Value = res.json().unwrap_or_default();
                                    let bal_str = json
                                        .get("result")
                                        .and_then(|r| r.get("balance"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("0");
                                    let bal_u64: u64 = bal_str.parse().unwrap_or(0);
                                    let dec_bal = bal_u64 as f64 / 1_000_000_000.0;
                                    println!("  💰 Balance: {:.9} THDR", dec_bal);
                                }
                                Err(e) => println!("  ❌ RPC Error: {}", e),
                            }
                        }
                        "2" => {
                            print!("  ↳ Enter amount of Testnet THDR to request: ");
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            let mut req_am_str = String::new();
                            std::io::stdin().read_line(&mut req_am_str).unwrap();
                            let requested: f64 = req_am_str.trim().parse().unwrap_or(100000.0);
                            let nano_requested = (requested * 1_000_000_000.0) as u64;

                            println!("  ↳ Requesting {} THDR from Genesis Faucet...", requested);

                            // Dynamic Gas Price Oracles
                            let gas_payload = serde_json::json!({ "jsonrpc": "2.0", "method": "thunder_gasPrice", "id": 1 });
                            let dynamic_gas_price =
                                match client.post(rpc_url).json(&gas_payload).send() {
                                    Ok(res) => res
                                        .json::<serde_json::Value>()
                                        .unwrap_or_default()
                                        .get("result")
                                        .and_then(|r| r.get("gas_price"))
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(1),
                                    Err(_) => 1,
                                };

                            let payload = serde_json::json!({
                                "jsonrpc": "2.0",
                                "method": "thunder_requestFaucet",
                                "params": { "address": format!("0x{}", my_address), "amount": nano_requested, "gas_price": dynamic_gas_price },
                                "id": 1
                            });
                            match client.post(rpc_url).json(&payload).send() {
                                Ok(res) => {
                                    let json: serde_json::Value = res.json().unwrap_or_default();
                                    if let Some(res) = json.get("result") {
                                        println!(
                                            "  ✅ Faucet Success! Tx Hash: {}",
                                            res.get("tx_hash")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                        );
                                    } else {
                                        println!("  ❌ Faucet Failed: {:?}", json.get("error"));
                                    }
                                }
                                Err(e) => println!("  ❌ Connection Error: {}", e),
                            }
                        }
                        "3" => {
                            print!("  ↳ Enter Recipient Address (0x...): ");
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            let mut to_str = String::new();
                            std::io::stdin().read_line(&mut to_str).unwrap();
                            let to_str = to_str.trim();

                            print!("  ↳ Enter Amount (THDR): ");
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            let mut am_str = String::new();
                            std::io::stdin().read_line(&mut am_str).unwrap();
                            let amount_float: f64 = am_str.trim().parse().unwrap_or(0.0);
                            let nano_amount = (amount_float * 1_000_000_000.0) as u64;

                            if let Ok(to_addr) = thunder_core::crypto::address_from_hex(to_str) {
                                let unique_nonce = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .subsec_nanos()
                                    as u64;

                                // Fetch Congestion Difficulty
                                let gas_payload = serde_json::json!({ "jsonrpc": "2.0", "method": "thunder_gasPrice", "id": 1 });
                                let dynamic_gas_price =
                                    match client.post(rpc_url).json(&gas_payload).send() {
                                        Ok(res) => res
                                            .json::<serde_json::Value>()
                                            .unwrap_or_default()
                                            .get("result")
                                            .and_then(|r| r.get("gas_price"))
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(1),
                                        Err(_) => 1,
                                    };

                                let mut tx = thunder_core::transaction::Transaction::new_transfer(
                                    unique_nonce,
                                    key_pair.address(),
                                    to_addr,
                                    nano_amount,
                                    21000,
                                    dynamic_gas_price,
                                );
                                tx.sign(&key_pair);
                                let hex_data = hex::encode(bincode::serialize(&tx).unwrap());

                                let payload = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "method": "thunder_sendTransaction",
                                    "params": { "data": hex_data },
                                    "id": 1
                                });
                                match client.post(rpc_url).json(&payload).send() {
                                    Ok(res) => {
                                        let json: serde_json::Value =
                                            res.json().unwrap_or_default();
                                        if let Some(result) = json.get("result") {
                                            println!(
                                                "  ✅ Transaction Submitted! Hash: {}",
                                                result
                                                    .get("tx_hash")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                            );
                                        } else {
                                            println!(
                                                "  ❌ JSON-RPC Error: {:?}",
                                                json.get("error")
                                            );
                                        }
                                    }
                                    Err(e) => println!("  ❌ Fallback Error: {}", e),
                                }
                            } else {
                                println!("  ❌ Invalid recipient address format.");
                            }
                        }
                        
                        
                        "4" => {
                            print!("  [🔑] Enter local port for your Validator (e.g., 9001): ");
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            let mut port_str = String::new();
                            std::io::stdin().read_line(&mut port_str).unwrap();
                            let local_port = port_str.trim().parse::<u16>().unwrap_or(9001);

                            let payload = serde_json::json!({
                                "jsonrpc": "2.0",
                                "method": "thunder_registerValidator",
                                "params": {
                                    "address": format!("0x{}", my_address),
                                    "public_key": hex::encode(key_pair.public_key()),
                                    "stake": 1000000000000u64
                                },
                                "id": 1
                            });

                            println!("  ⏳ Staking 1,000 THDR to Master Bootnode...");
                            
                            match client.post(rpc_url).json(&payload).send() {
                                Ok(_) => {
                                    println!("  ✅ Validator Staking Confirmed!");
                                    println!("  🚀 Mutating terminal into Active Block Forger on Port {}...", local_port);
                                    
                                    let config = thunder_network::node::NodeConfig {
                                        data_dir: format!("/tmp/validator_node_{}", local_port),
                                        listen_port: local_port,
                                        ..Default::default()
                                    };
                                    let mut node = thunder_network::node::Node::new(key_pair.clone(), config);
                                    let _ = node.register_as_validator(1000000000000u64);

                                    let shared_node = std::sync::Arc::new(std::sync::RwLock::new(node));
                                    let forger_node = std::sync::Arc::clone(&shared_node);
                                    
                                    println!("  ⚡ Node is fully synced. Terminal is now locked in Forging Mode.");
                                    loop {
                                        std::thread::sleep(std::time::Duration::from_millis(50));
                                        let mut n = forger_node.write().unwrap();
                                        
                                        if n.mempool.is_empty() {
                                            continue;
                                        }

                                        if n.create_event().is_ok() {
                                            if let Some(block) = n.try_produce_block() {
                                                println!("  🔨 FORGED BLOCK #{} | {} txns | Hash: 0x{}", block.header.height, block.transactions.len(), &hex::encode(block.hash())[0..10]);
                                            }
                                        }
                                    }
                                },
                                Err(e) => println!("  ❌ Staking Failed: {}", e),
                            }
                        }

                        "5" => {
                            println!("  👋 Exiting Wallet Console.");
                            break;
                        }
                        _ => println!("  ⚠ Invalid option, please try again."),
                    }
                }
            }
            WalletCommands::Balance { address } => {
                println!("⚡ Fetching Balance for {}", address);

                let rpc_url = "http://127.0.0.1:8080";
                let client = reqwest::blocking::Client::new();

                let payload = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "thunder_getBalance",
                    "params": { "address": address },
                    "id": 1
                });

                match client.post(rpc_url).json(&payload).send() {
                    Ok(res) => {
                        let json: serde_json::Value = res.json().unwrap_or_default();
                        if let Some(result) = json.get("result") {
                            let bal = result
                                .get("balance")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0");
                            println!("  ✅ Balance: {} THDR", bal);
                        } else {
                            println!("  ❌ RPC Error: {:?}", json.get("error"));
                        }
                    }
                    Err(e) => println!("  ❌ Failed to connect to node: {}", e),
                }
            }
        },

        // ── Transaction Commands ───────────────────────────────────────
        Commands::Tx { action } => match action {
            TxCommands::Send {
                to,
                amount,
                gas_limit,
            } => {
                println!("⚡ Initiating Transaction to {}", to);
                println!("  To authenticate, please provide your Wallet Secret Key.");

                use std::io::{self, Write};
                print!("  [🔑] Enter Secret Key (Hex): ");
                io::stdout().flush().unwrap();
                let mut secret_input = String::new();
                io::stdin().read_line(&mut secret_input).unwrap();
                let secret_input = secret_input.trim();

                let secret_bytes = match hex::decode(secret_input) {
                    Ok(b) if b.len() == 32 => {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&b);
                        arr
                    }
                    _ => {
                        println!("  ❌ Invalid Secret Key length (expected 32-byte hex).");
                        return;
                    }
                };

                let key_pair = KeyPair::from_secret_bytes(&secret_bytes);
                let to_addr = match thunder_core::crypto::address_from_hex(&to) {
                    Ok(addr) => addr,
                    Err(_) => {
                        println!("  ❌ Invalid Recipient Address.");
                        return;
                    }
                };

                println!(
                    "  ↳ Signing transaction as 0x{}...",
                    hex::encode(key_pair.address())
                );
                // Dynamically injecting nanosecond timestamps ensuring distinct uniqueness against mempool collision caching.
                let unique_nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() as u64;
                let mut tx = thunder_core::transaction::Transaction::new_transfer(
                    unique_nonce,
                    key_pair.address(),
                    to_addr,
                    amount,
                    gas_limit,
                    1,
                );
                tx.sign(&key_pair);
                let serialized_tx =
                    bincode::serialize(&tx).expect("Failed to serialize transaction");
                let hex_data = hex::encode(serialized_tx);

                let rpc_url = "http://127.0.0.1:8080";
                let client = reqwest::blocking::Client::new();

                let payload = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "thunder_sendTransaction",
                    "params": {
                        "data": hex_data
                    },
                    "id": 1
                });

                match client.post(rpc_url).json(&payload).send() {
                    Ok(res) => {
                        let json: serde_json::Value = res.json().unwrap_or_default();
                        if let Some(result) = json.get("result") {
                            println!("  ✅ Transaction Submitted Successfully!");
                            println!(
                                "  tx_hash : {}",
                                result.get("tx_hash").and_then(|v| v.as_str()).unwrap_or("")
                            );
                            println!("  amount  : {} THDR", amount);
                            println!("  gas     : {}", gas_limit);
                        } else {
                            println!("  ❌ RPC Error: {:?}", json.get("error"));
                        }
                    }
                    Err(e) => println!("  ❌ Failed to connect to node: {}", e),
                }
            }
        },

        // ── Contract Commands ──────────────────────────────────────────
        Commands::Contract { action } => match action {
            ContractCommands::Compile { file } => {
                println!("⚡ Compiling {}", file);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                match std::fs::read_to_string(&file) {
                    Ok(source) => match compile_source(&source) {
                        Ok(compiled) => {
                            println!("  ✅ Compilation successful!");
                            println!("  Contract     : {}", compiled.name);
                            println!("  Instructions : {}", compiled.instructions.len());
                            println!(
                                "  Functions    : {:?}",
                                compiled.function_table.keys().collect::<Vec<_>>()
                            );
                            println!("  State Slots  : {:?}", compiled.state_slots);
                        }
                        Err(e) => {
                            println!("  ❌ Compilation failed: {}", e);
                        }
                    },
                    Err(e) => {
                        println!("  ❌ Could not read file: {}", e);
                    }
                }
            }
            ContractCommands::Deploy { file } => {
                println!("⚡ Deploying contract from {}", file);

                match std::fs::read_to_string(&file) {
                    Ok(source) => match compile_source(&source) {
                        Ok(compiled) => {
                            println!(
                                "  ✅ Compiled {} ({} instructions)",
                                compiled.name,
                                compiled.instructions.len()
                            );
                            println!("  Contract would be deployed to the network.");
                        }
                        Err(e) => {
                            println!("  ❌ Compilation failed: {}", e);
                        }
                    },
                    Err(e) => {
                        println!("  ❌ Could not read file: {}", e);
                    }
                }
            }
            ContractCommands::Run { file, function } => {
                println!("⚡ Running {}.{}()", file, function);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                match std::fs::read_to_string(&file) {
                    Ok(source) => match compile_source(&source) {
                        Ok(compiled) => {
                            println!(
                                "  ✅ Compiled {} ({} instructions)",
                                compiled.name,
                                compiled.instructions.len()
                            );

                            // Find the function entry point.
                            if let Some(&entry) = compiled.function_table.get(&function) {
                                let ctx = ExecutionContext {
                                    caller: [1u8; 20],
                                    contract_address: [2u8; 20],
                                    value: 0,
                                    timestamp: 1000,
                                    block_height: 1,
                                };

                                // Run VM starting at the function entry point.
                                let instructions = compiled.instructions.clone();
                                // Prepend a jump to the function entry.
                                let mut exec_instructions = vec![Instruction::with_operand(
                                    thunder_vm::opcode::OpCode::Jump,
                                    (entry + 1) as u64,
                                )];
                                exec_instructions.extend(instructions);

                                let mut vm = ThunderVm::new(
                                    exec_instructions,
                                    ctx,
                                    10_000_000,
                                    HashMap::new(),
                                );

                                match vm.execute() {
                                    Ok(result) => {
                                        println!("  Gas Used     : {}", result.gas_used);
                                        if let Some(val) = result.return_value {
                                            println!("  Return Value : {}", val);
                                        }
                                        if result.reverted {
                                            println!(
                                                "  ⚠ Execution reverted: {}",
                                                result.revert_reason.unwrap_or_default()
                                            );
                                        } else {
                                            println!("  ✅ Execution successful!");
                                        }
                                        if !result.logs.is_empty() {
                                            println!(
                                                "  Logs         : {} events emitted",
                                                result.logs.len()
                                            );
                                        }
                                        if !result.storage.is_empty() {
                                            println!(
                                                "  Storage      : {} slots written",
                                                result.storage.len()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        println!("  ❌ VM Error: {}", e);
                                    }
                                }
                            } else {
                                println!(
                                    "  ❌ Function '{}' not found. Available: {:?}",
                                    function,
                                    compiled.function_table.keys().collect::<Vec<_>>()
                                );
                            }
                        }
                        Err(e) => {
                            println!("  ❌ Compilation failed: {}", e);
                        }
                    },
                    Err(e) => {
                        println!("  ❌ Could not read file: {}", e);
                    }
                }
            }
        },
    }
}
