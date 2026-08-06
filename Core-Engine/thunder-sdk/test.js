"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const wallet_1 = require("./src/wallet");
const transaction_1 = require("./src/transaction");
function runTests() {
    console.log("=== Thunder SDK Tests ===");
    // 1. Test Wallet Generation
    const wallet = new wallet_1.Wallet();
    console.log("New Wallet generated!");
    console.log("Address:", wallet.getAddressHex());
    // 2. Test Transaction Serialization & Signing
    const toWallet = new wallet_1.Wallet();
    const tx = new transaction_1.Transaction(1n, // chain_id
    0n, // nonce
    wallet.address, toWallet.address, 1000n, // value
    new Uint8Array(0), // data
    21000n, // gas_limit
    10n, // gas_price
    transaction_1.TransactionKind.Transfer);
    console.log("\nSigning Transaction...");
    const signedTx = wallet.signTransaction(tx);
    const serialized = signedTx.serialize();
    console.log("Serialized Transaction Length:", serialized.length, "bytes");
    const toHex = (arr) => Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join('');
    console.log("Serialized Transaction Hex:", toHex(serialized));
    console.log("\nTests passed successfully.");
}
runTests();
