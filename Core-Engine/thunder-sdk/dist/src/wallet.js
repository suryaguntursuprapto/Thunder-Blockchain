"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Wallet = void 0;
const tweetnacl_1 = __importDefault(require("tweetnacl"));
const js_sha256_1 = require("js-sha256");
class Wallet {
    /**
     * Initialize a wallet from a 32-byte seed/secret key
     * @param seed 32 bytes of raw secret key
     */
    constructor(seed) {
        let keyPair;
        if (seed) {
            if (seed.length !== 32)
                throw new Error("Seed must be exactly 32 bytes");
            keyPair = tweetnacl_1.default.sign.keyPair.fromSeed(seed);
        }
        else {
            keyPair = tweetnacl_1.default.sign.keyPair();
        }
        this.publicKey = keyPair.publicKey;
        this.secretKey = keyPair.secretKey;
        // Address is the first 20 bytes of SHA256(publicKey)
        const hash = js_sha256_1.sha256.create();
        hash.update(this.publicKey);
        const hashBytes = new Uint8Array(hash.array());
        this.address = hashBytes.slice(0, 20);
    }
    /**
     * Hex string of the 20-byte address (useful for RPC)
     */
    getAddressHex() {
        return Array.from(this.address)
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
    }
    /**
     * Sign a transaction object, populating its signature and public_key fields.
     */
    signTransaction(tx) {
        // Rust's hash_to_hex is just a sha256 hash.
        const serialized = tx.serializeForSignature();
        // Compute transaction hash (SHA256)
        const hash = js_sha256_1.sha256.create();
        hash.update(serialized);
        const txHash = new Uint8Array(hash.array());
        // Sign the hash
        const signature = tweetnacl_1.default.sign.detached(txHash, this.secretKey);
        tx.signature = signature;
        tx.public_key = this.publicKey;
        tx.from = this.address;
        return tx;
    }
}
exports.Wallet = Wallet;
