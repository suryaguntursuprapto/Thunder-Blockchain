"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Transaction = exports.TransactionKind = void 0;
const bincode_1 = require("./bincode");
var TransactionKind;
(function (TransactionKind) {
    TransactionKind[TransactionKind["Transfer"] = 0] = "Transfer";
    TransactionKind[TransactionKind["ContractDeploy"] = 1] = "ContractDeploy";
    TransactionKind[TransactionKind["ContractCall"] = 2] = "ContractCall";
    TransactionKind[TransactionKind["Stake"] = 3] = "Stake";
    TransactionKind[TransactionKind["Unstake"] = 4] = "Unstake";
})(TransactionKind || (exports.TransactionKind = TransactionKind = {}));
class Transaction {
    constructor(chain_id, nonce, from, to, value, data, gas_limit, gas_price, kind, signature = new Uint8Array(64), public_key = new Uint8Array(32)) {
        this.chain_id = chain_id;
        this.nonce = nonce;
        this.from = from;
        this.to = to;
        this.value = value;
        this.data = data;
        this.gas_limit = gas_limit;
        this.gas_price = gas_price;
        this.kind = kind;
        this.signature = signature;
        this.public_key = public_key;
        if (from.length !== 20)
            throw new Error("From address must be 20 bytes");
        if (to.length !== 20)
            throw new Error("To address must be 20 bytes");
    }
    /**
     * Serialize the transaction to Bincode format (matching Rust)
     * Excludes signature and public_key for signing (hash generation)
     */
    serializeForSignature() {
        const ser = new bincode_1.BincodeSerializer();
        ser.writeU64(this.chain_id);
        ser.writeU64(this.nonce);
        ser.writeBytes(this.from);
        ser.writeBytes(this.to);
        ser.writeU64(this.value);
        ser.writeVec(this.data);
        ser.writeU64(this.gas_limit);
        ser.writeU64(this.gas_price);
        ser.writeU32(this.kind);
        return ser.getBytes();
    }
    /**
     * Fully serialize the transaction including signature and public key
     */
    serialize() {
        const ser = new bincode_1.BincodeSerializer();
        ser.writeU64(this.chain_id);
        ser.writeU64(this.nonce);
        ser.writeBytes(this.from);
        ser.writeBytes(this.to);
        ser.writeU64(this.value);
        ser.writeVec(this.data);
        ser.writeU64(this.gas_limit);
        ser.writeU64(this.gas_price);
        ser.writeU32(this.kind);
        // BigArray signature (64 bytes)
        ser.writeBytes(this.signature);
        // public_key (32 bytes)
        ser.writeBytes(this.public_key);
        return ser.getBytes();
    }
}
exports.Transaction = Transaction;
