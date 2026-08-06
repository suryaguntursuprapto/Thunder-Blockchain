import { ThunderProvider } from './provider';
import { Wallet } from './wallet';
export declare class Contract {
    readonly addressHex: string;
    readonly provider: ThunderProvider;
    readonly wallet?: Wallet | undefined;
    /**
     * @param addressHex The 20-byte address (in hex) of the deployed contract
     * @param provider The provider to use for network queries
     * @param wallet (Optional) The wallet to use for signing state-modifying calls
     */
    constructor(addressHex: string, provider: ThunderProvider, wallet?: Wallet | undefined);
    /**
     * Helper to convert a hex string to Uint8Array
     */
    private hexToBytes;
    /**
     * Deploy a new contract to the network.
     * @param compiledBytecodeHex The hex string returned by compileContract
     * @param gasLimit Maximum gas for deployment
     * @param gasPrice Gas price
     * @returns The transaction hash of the deployment
     */
    static deploy(provider: ThunderProvider, wallet: Wallet, compiledBytecodeHex: string, gasLimit?: bigint, gasPrice?: bigint): Promise<string>;
    /**
     * Call a read-only method on the contract (no state changes).
     * @param dataHex The call data in hex (e.g., serialized arguments)
     */
    read(dataHex: string): Promise<string>;
    /**
     * Send a state-modifying transaction to the contract.
     * @param dataHex The call data in hex
     * @param value Amount of coins to send with the call
     */
    send(dataHex: string, value?: bigint, gasLimit?: bigint, gasPrice?: bigint): Promise<string>;
}
