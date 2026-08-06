export declare enum TransactionKind {
    Transfer = 0,
    ContractDeploy = 1,
    ContractCall = 2,
    Stake = 3,
    Unstake = 4
}
export declare class Transaction {
    chain_id: bigint;
    nonce: bigint;
    from: Uint8Array;
    to: Uint8Array;
    value: bigint;
    data: Uint8Array;
    gas_limit: bigint;
    gas_price: bigint;
    kind: TransactionKind;
    signature: Uint8Array;
    public_key: Uint8Array;
    constructor(chain_id: bigint, nonce: bigint, from: Uint8Array, to: Uint8Array, value: bigint, data: Uint8Array, gas_limit: bigint, gas_price: bigint, kind: TransactionKind, signature?: Uint8Array, public_key?: Uint8Array);
    /**
     * Serialize the transaction to Bincode format (matching Rust)
     * Excludes signature and public_key for signing (hash generation)
     */
    serializeForSignature(): Uint8Array;
    /**
     * Fully serialize the transaction including signature and public key
     */
    serialize(): Uint8Array;
}
