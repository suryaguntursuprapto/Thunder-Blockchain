import { Transaction } from './transaction';
export declare class ThunderProvider {
    readonly rpcUrl: string;
    constructor(rpcUrl: string);
    private rpcCall;
    getChainId(): Promise<number>;
    getBalance(addressHex: string): Promise<number>;
    getNonce(addressHex: string): Promise<number>;
    sendTransaction(tx: Transaction): Promise<string>;
    call(addressHex: string, publicKeyHex: string, dataHex: string): Promise<string>;
    compileContract(source: string): Promise<string>;
}
