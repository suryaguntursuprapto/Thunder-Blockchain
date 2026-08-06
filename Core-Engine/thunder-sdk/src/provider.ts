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
        const res = await this.rpcCall<any>('thunder_chainId');
        return res.chain_id ? Number(res.chain_id) : Number(res);
    }

    async getBalance(addressHex: string): Promise<number> {
        const res = await this.rpcCall<any>('thunder_getBalance', { address: addressHex });
        return res.balance ? Number(res.balance) : 0;
    }

    async getNonce(addressHex: string): Promise<number> {
        const res = await this.rpcCall<any>('thunder_getNonce', { address: addressHex });
        return res.nonce ? Number(res.nonce) : 0;
    }

    async sendTransaction(tx: Transaction): Promise<string> {
        const serialized = tx.serialize();
        const hex = Array.from(serialized)
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
        
        // server.rs expects 'data' not 'tx'
        return await this.rpcCall<string>('thunder_sendTransaction', { data: hex });
    }

    async call(addressHex: string, publicKeyHex: string, dataHex: string): Promise<string> {
        return await this.rpcCall<string>('thunder_call', {
            address: addressHex,
            public_key: publicKeyHex,
            data: dataHex
        });
    }

    async compileContract(source: string): Promise<string> {
        const res = await this.rpcCall<any>('thunder_compileContract', { source });
        return res.bytecode ? res.bytecode : res;
    }
}
