import axios from 'axios';
import { Transaction } from './transaction';

export class ThunderProvider {
    constructor(public readonly rpcUrl: string) {}

    private async rpcCall<T>(method: string, params: any = {}): Promise<T> {
        const response = await axios.post(this.rpcUrl, {
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

    async getChainId(): Promise<number> {
        return await this.rpcCall<number>('thunder_chainId');
    }

    async getBalance(addressHex: string): Promise<number> {
        return await this.rpcCall<number>('thunder_balance', { address: addressHex });
    }

    async getNonce(addressHex: string): Promise<number> {
        return await this.rpcCall<number>('thunder_nonce', { address: addressHex });
    }

    async sendTransaction(tx: Transaction): Promise<string> {
        const serialized = tx.serialize();
        const hex = Array.from(serialized)
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
        
        return await this.rpcCall<string>('thunder_sendTransaction', { tx: hex });
    }

    async call(addressHex: string, publicKeyHex: string, dataHex: string): Promise<string> {
        return await this.rpcCall<string>('thunder_call', {
            address: addressHex,
            public_key: publicKeyHex,
            data: dataHex
        });
    }

    async compileContract(source: string): Promise<string> {
        return await this.rpcCall<string>('thunder_compileContract', { source });
    }
}
