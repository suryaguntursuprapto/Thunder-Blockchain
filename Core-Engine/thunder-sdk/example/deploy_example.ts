import { ThunderProvider, Wallet, Contract } from '../src';
import * as fs from 'fs';
import * as path from 'path';

async function run() {
    console.log("⚡ Starting Thunder SDK Example ⚡");

    // 1. Connect to the local running node (ensure it's running via `cargo run -p thunder-cli -- --node`)
    const provider = new ThunderProvider("http://127.0.0.1:8085");

    try {
        const chainId = await provider.getChainId();
        console.log("Connected to Network. Chain ID:", chainId);
    } catch (e) {
        console.error("❌ Failed to connect to node. Make sure the Thunder Node is running at http://127.0.0.1:8085");
        return;
    }

    // 2. Generate the Genesis Wallet (which has 1B THDR)
    const seed = new Uint8Array(32);
    seed[31] = 1; // 000...001
    const wallet = new Wallet(seed);
    console.log("🔑 Developer Wallet Address:", wallet.getAddressHex());

    // 3. Load the ThunderScript file
    const sourceCode = fs.readFileSync(path.join(__dirname, '../../example/Counter.ths'), 'utf8');
    console.log("📄 Source Code Loaded:\n", sourceCode);

    // 4. Compile the Contract via RPC
    console.log("⚙️ Compiling Contract...");
    let compiledBytecode: string;
    try {
        compiledBytecode = await provider.compileContract(sourceCode);
        console.log("✅ Contract Compiled Successfully! Bytecode Length:", compiledBytecode.length / 2, "bytes");
    } catch (e: any) {
        console.error("❌ Compilation failed:", e.message);
        return;
    }

    // 5. Deploy the Contract
    console.log("🚀 Deploying Contract to the blockchain...");
    try {
        const txHash = await Contract.deploy(provider, wallet, compiledBytecode, 100000n, 1n);
        console.log("✅ Deployment Transaction Sent! TxHash:", txHash);
        
        // Normally we'd wait for consensus here, but since this is an example:
        console.log("   (Wait a few seconds for the block to be mined...)");

    } catch (e: any) {
        console.error("❌ Deployment failed:", e.message);
        console.error("   Note: The node might require you to have a positive balance to deploy.");
    }
}

run();
