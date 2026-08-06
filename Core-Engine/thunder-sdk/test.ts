import { Wallet } from './src/wallet';
import { Transaction, TransactionKind } from './src/transaction';

function runTests() {
    console.log("=== Thunder SDK Tests ===");

    // 1. Test Wallet Generation
    const wallet = new Wallet();
    console.log("New Wallet generated!");
    console.log("Address:", wallet.getAddressHex());

    // 2. Test Transaction Serialization & Signing
    const toWallet = new Wallet();
    const tx = new Transaction(
        1n, // chain_id
        0n, // nonce
        wallet.address,
        toWallet.address,
        1000n, // value
        new Uint8Array(0), // data
        21000n, // gas_limit
        10n, // gas_price
        TransactionKind.Transfer
    );

    console.log("\nSigning Transaction...");
    const signedTx = wallet.signTransaction(tx);
    
    const serialized = signedTx.serialize();
    console.log("Serialized Transaction Length:", serialized.length, "bytes");

    const toHex = (arr: Uint8Array) => Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join('');
    console.log("Serialized Transaction Hex:", toHex(serialized));

    console.log("\nTests passed successfully.");
}

runTests();
