"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BincodeSerializer = void 0;
class BincodeSerializer {
    constructor(initialSize = 1024) {
        this.buffer = new Uint8Array(initialSize);
        this.offset = 0;
    }
    ensureCapacity(size) {
        if (this.offset + size > this.buffer.length) {
            const newBuffer = new Uint8Array(Math.max(this.buffer.length * 2, this.offset + size));
            newBuffer.set(this.buffer);
            this.buffer = newBuffer;
        }
    }
    writeU64(value) {
        this.ensureCapacity(8);
        const dataView = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
        dataView.setBigUint64(this.offset, value, true); // true = little endian
        this.offset += 8;
    }
    writeU32(value) {
        this.ensureCapacity(4);
        const dataView = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
        dataView.setUint32(this.offset, value, true); // true = little endian
        this.offset += 4;
    }
    writeBytes(bytes) {
        this.ensureCapacity(bytes.length);
        this.buffer.set(bytes, this.offset);
        this.offset += bytes.length;
    }
    writeVec(bytes) {
        // Bincode uses u64 for length prefix
        this.writeU64(BigInt(bytes.length));
        this.writeBytes(bytes);
    }
    getBytes() {
        return this.buffer.slice(0, this.offset);
    }
}
exports.BincodeSerializer = BincodeSerializer;
