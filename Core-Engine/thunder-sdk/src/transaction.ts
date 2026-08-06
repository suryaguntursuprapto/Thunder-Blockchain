import { BincodeSerializer } from './bincode';

export enum TransactionKind {
    Transfer = 0,
    ContractDeploy = 1,
    ContractCall = 2,
    Stake = 3,
    Unstake = 4,
}

export class Transaction {
    constructor(
        public chain_id: bigint,
        public nonce: bigint,
        public from: Uint8Array,
        public to: Uint8Array,
        public value: bigint,
        public data: Uint8Array,
        public gas_limit: bigint,
        public gas_price: bigint,
        public kind: TransactionKind,
        public signature: Uint8Array = new Uint8Array(64),
        public public_key: Uint8Array = new Uint8Array(32)
    ) {
        if (from.length !== 20) throw new Error("From address must be 20 bytes");
        if (to.length !== 20) throw new Error("To address must be 20 bytes");
    }

    /**
     * Serialize the transaction to Bincode format (matching Rust)
     * Excludes signature and public_key for signing (hash generation)
     */
    serializeForSignature(): Uint8Array {
        const ser = new BincodeSerializer();
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
    serialize(): Uint8Array {
        const ser = new BincodeSerializer();
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
