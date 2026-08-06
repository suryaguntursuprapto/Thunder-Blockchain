import { Transaction } from './transaction';
export declare class Wallet {
    readonly publicKey: Uint8Array;
    readonly secretKey: Uint8Array;
    readonly address: Uint8Array;
    /**
     * Initialize a wallet from a 32-byte seed/secret key
     * @param seed 32 bytes of raw secret key
     */
    constructor(seed?: Uint8Array);
    /**
     * Hex string of the 20-byte address (useful for RPC)
     */
    getAddressHex(): string;
    /**
     * Sign a transaction object, populating its signature and public_key fields.
     */
    signTransaction(tx: Transaction): Transaction;
}
