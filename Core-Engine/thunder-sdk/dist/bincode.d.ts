export declare class BincodeSerializer {
    private buffer;
    private offset;
    constructor(initialSize?: number);
    private ensureCapacity;
    writeU64(value: bigint): void;
    writeU32(value: number): void;
    writeBytes(bytes: Uint8Array): void;
    writeVec(bytes: Uint8Array): void;
    getBytes(): Uint8Array;
}
