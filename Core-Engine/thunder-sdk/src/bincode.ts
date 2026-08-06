export class BincodeSerializer {
    private buffer: Uint8Array;
    private offset: number;

    constructor(initialSize = 1024) {
        this.buffer = new Uint8Array(initialSize);
        this.offset = 0;
    }

    private ensureCapacity(size: number) {
        if (this.offset + size > this.buffer.length) {
            const newBuffer = new Uint8Array(Math.max(this.buffer.length * 2, this.offset + size));
            newBuffer.set(this.buffer);
            this.buffer = newBuffer;
        }
    }

    writeU64(value: bigint) {
        this.ensureCapacity(8);
        const dataView = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
        dataView.setBigUint64(this.offset, value, true); // true = little endian
        this.offset += 8;
    }

    writeU32(value: number) {
        this.ensureCapacity(4);
        const dataView = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
        dataView.setUint32(this.offset, value, true); // true = little endian
        this.offset += 4;
    }

    writeBytes(bytes: Uint8Array) {
        this.ensureCapacity(bytes.length);
        this.buffer.set(bytes, this.offset);
        this.offset += bytes.length;
    }

    writeVec(bytes: Uint8Array) {
        // Bincode uses u64 for length prefix
        this.writeU64(BigInt(bytes.length));
        this.writeBytes(bytes);
    }

    getBytes(): Uint8Array {
        return this.buffer.slice(0, this.offset);
    }
}
