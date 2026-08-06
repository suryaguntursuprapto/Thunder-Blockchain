"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const src_1 = require("../src");
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
async function run() {
    console.log("⚡ Starting Thunder SDK Example ⚡");
    // 1. Connect to the local running node (ensure it's running via `cargo run -p thunder-cli -- --node`)
    const provider = new src_1.ThunderProvider("http://127.0.0.1:8080");
    try {
        const chainId = await provider.getChainId();
        console.log("Connected to Network. Chain ID:", chainId);
    }
    catch (e) {
        console.error("❌ Failed to connect to node. Make sure the Thunder Node is running at http://127.0.0.1:8080");
        return;
    }
    // 2. Generate a new Developer Wallet
    const wallet = new src_1.Wallet();
    console.log("🔑 Developer Wallet Address:", wallet.getAddressHex());
    // 3. Load the ThunderScript file
    const sourceCode = fs.readFileSync(path.join(__dirname, 'Counter.ths'), 'utf8');
    console.log("📄 Source Code Loaded:\n", sourceCode);
    // 4. Compile the Contract via RPC
    console.log("⚙️ Compiling Contract...");
    let compiledBytecode;
    try {
        compiledBytecode = await provider.compileContract(sourceCode);
        console.log("✅ Contract Compiled Successfully! Bytecode Length:", compiledBytecode.length / 2, "bytes");
    }
    catch (e) {
        console.error("❌ Compilation failed:", e.message);
        return;
    }
    // 5. Deploy the Contract
    console.log("🚀 Deploying Contract to the blockchain...");
    try {
        const txHash = await src_1.Contract.deploy(provider, wallet, compiledBytecode, 100000n, 1n);
        console.log("✅ Deployment Transaction Sent! TxHash:", txHash);
        // Normally we'd wait for consensus here, but since this is an example:
        console.log("   (Wait a few seconds for the block to be mined...)");
    }
    catch (e) {
        console.error("❌ Deployment failed:", e.message);
        console.error("   Note: The node might require you to have a positive balance to deploy.");
    }
}
run();
