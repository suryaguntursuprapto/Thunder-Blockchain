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

                let key_pair = KeyPair::generate();
                println!("  Node Address : {}", address_to_hex(&key_pair.address()));
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
                node.register_as_validator(100_000)
                    .expect("failed to register as validator");

                // Give the node's account some initial coins.
                node.state.set_account(
                    &node.key_pair.address(),
                    Account::with_balance(1_000_000_000),
                );

                println!("  Status       : ✅ Node initialised (genesis block created)");
                println!("  Validators   : {}", node.validator_set.active_count());
                println!("  Chain Height : {}", node.height());
                println!();
                println!("  Node is ready. In a full deployment, the P2P event");
                println!("  loop would start here using libp2p + tokio.");

                // Mount the HTTP JSON-RPC Server
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    thunder_rpc::start_server(8080).await;
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
            WalletCommands::Balance { address } => {
                println!("⚡ Balance for {}", address);
                println!("  Balance: 0 THDR");
            }
        },

        // ── Transaction Commands ───────────────────────────────────────
        Commands::Tx { action } => match action {
            TxCommands::Send {
                to,
                amount,
                gas_limit,
            } => {
                println!("⚡ Sending {} THDR to {}", amount, to);
                println!("  Gas Limit: {}", gas_limit);
                println!("  Status: Transaction would be broadcast to network.");
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
