import nacl from 'tweetnacl';
import { sha256 } from 'js-sha256';
import { Transaction } from './transaction';

export class Wallet {
    public readonly publicKey: Uint8Array;
    public readonly secretKey: Uint8Array;
    public readonly address: Uint8Array;

    /**
     * Initialize a wallet from a 32-byte seed/secret key
     * @param seed 32 bytes of raw secret key
     */
    constructor(seed?: Uint8Array) {
        let keyPair: nacl.SignKeyPair;
        if (seed) {
            if (seed.length !== 32) throw new Error("Seed must be exactly 32 bytes");
            keyPair = nacl.sign.keyPair.fromSeed(seed);
        } else {
            keyPair = nacl.sign.keyPair();
        }

        this.publicKey = keyPair.publicKey;
        this.secretKey = keyPair.secretKey;

        // Address is the first 20 bytes of SHA256(publicKey)
        const hash = sha256.create();
        hash.update(this.publicKey);
        const hashBytes = new Uint8Array(hash.array());
        this.address = hashBytes.slice(0, 20);
    }

    /**
     * Hex string of the 20-byte address (useful for RPC)
     */
    getAddressHex(): string {
        return Array.from(this.address)
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
    }

    /**
     * Sign a transaction object, populating its signature and public_key fields.
     */
    signTransaction(tx: Transaction): Transaction {
        // Rust's hash_to_hex is just a sha256 hash.
        const serialized = tx.serializeForSignature();
        
        // Compute transaction hash (SHA256)
        const hash = sha256.create();
        hash.update(serialized);
        const txHash = new Uint8Array(hash.array());

        // Sign the hash
        const signature = nacl.sign.detached(txHash, this.secretKey);

        tx.signature = signature;
        tx.public_key = this.publicKey;
        tx.from = this.address;

        return tx;
    }
}
