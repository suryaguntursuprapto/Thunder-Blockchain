"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ThunderProvider = void 0;
const axios_1 = __importDefault(require("axios"));
const transaction_1 = require("./transaction");
class ThunderProvider {
    rpcUrl;
    constructor(rpcUrl) {
        this.rpcUrl = rpcUrl;
    }
    async rpcCall(method, params = {}) {
        const response = await axios_1.default.post(this.rpcUrl, {
            jsonrpc: "2.0",
            method,
            params,
            id: Date.now(),
        });
        if (response.data.error) {
            throw new Error(`RPC Error: ${response.data.error.message}`);
        }
        return response.data.result;
    }
    async getChainId() {
        return await this.rpcCall('thunder_chainId');
    }
    async getBalance(addressHex) {
        return await this.rpcCall('thunder_balance', { address: addressHex });
    }
    async getNonce(addressHex) {
        return await this.rpcCall('thunder_nonce', { address: addressHex });
    }
    async sendTransaction(tx) {
        const serialized = tx.serialize();
        const hex = Array.from(serialized)
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
        return await this.rpcCall('thunder_sendTransaction', { tx: hex });
    }
    async call(addressHex, publicKeyHex, dataHex) {
        return await this.rpcCall('thunder_call', {
            address: addressHex,
            public_key: publicKeyHex,
            data: dataHex
        });
    }
    async compileContract(source) {
        return await this.rpcCall('thunder_compileContract', { source });
    }
}
exports.ThunderProvider = ThunderProvider;
//# sourceMappingURL=provider.js.map