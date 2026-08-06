"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Contract = void 0;
const transaction_1 = require("./transaction");
class Contract {
    /**
     * @param addressHex The 20-byte address (in hex) of the deployed contract
     * @param provider The provider to use for network queries
     * @param wallet (Optional) The wallet to use for signing state-modifying calls
     */
    constructor(addressHex, provider, wallet) {
        this.addressHex = addressHex;
        this.provider = provider;
        this.wallet = wallet;
    }
    /**
     * Helper to convert a hex string to Uint8Array
     */
    hexToBytes(hex) {
        if (hex.length % 2 !== 0)
            throw new Error("Invalid hex string");
        const bytes = new Uint8Array(hex.length / 2);
        for (let i = 0; i < hex.length; i += 2) {
            bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
        }
        return bytes;
    }
    /**
     * Deploy a new contract to the network.
     * @param compiledBytecodeHex The hex string returned by compileContract
     * @param gasLimit Maximum gas for deployment
     * @param gasPrice Gas price
     * @returns The transaction hash of the deployment
     */
    static async deploy(provider, wallet, compiledBytecodeHex, gasLimit = 100000n, gasPrice = 10n) {
        const chainId = BigInt(await provider.getChainId());
        const nonce = BigInt(await provider.getNonce(wallet.getAddressHex()));
        let dataBytes;
        try {
            // Remove '0x' prefix if exists
            let cleanHex = compiledBytecodeHex;
            if (cleanHex.startsWith('0x'))
                cleanHex = cleanHex.slice(2);
            const bytes = new Uint8Array(cleanHex.length / 2);
            for (let i = 0; i < cleanHex.length; i += 2) {
                bytes[i / 2] = parseInt(cleanHex.substring(i, i + 2), 16);
            }
            dataBytes = bytes;
        }
        catch (e) {
            throw new Error("Invalid compiled bytecode format");
        }
        const tx = new transaction_1.Transaction(chainId, nonce, wallet.address, new Uint8Array(20), // zero address for deployment
        0n, // value
        dataBytes, gasLimit, gasPrice, transaction_1.TransactionKind.ContractDeploy);
        wallet.signTransaction(tx);
        return await provider.sendTransaction(tx);
    }
    /**
     * Call a read-only method on the contract (no state changes).
     * @param dataHex The call data in hex (e.g., serialized arguments)
     */
    async read(dataHex) {
        let pubKey = "";
        if (this.wallet) {
            pubKey = Array.from(this.wallet.publicKey).map(b => b.toString(16).padStart(2, '0')).join('');
        }
        return await this.provider.call(this.addressHex, pubKey, dataHex);
    }
    /**
     * Send a state-modifying transaction to the contract.
     * @param dataHex The call data in hex
     * @param value Amount of coins to send with the call
     */
    async send(dataHex, value = 0n, gasLimit = 50000n, gasPrice = 10n) {
        if (!this.wallet)
            throw new Error("Wallet is required to send state-modifying transactions");
        const chainId = BigInt(await this.provider.getChainId());
        const nonce = BigInt(await this.provider.getNonce(this.wallet.getAddressHex()));
        const dataBytes = this.hexToBytes(dataHex);
        const toBytes = this.hexToBytes(this.addressHex);
        const tx = new transaction_1.Transaction(chainId, nonce, this.wallet.address, toBytes, value, dataBytes, gasLimit, gasPrice, transaction_1.TransactionKind.ContractCall);
        this.wallet.signTransaction(tx);
        return await this.provider.sendTransaction(tx);
    }
}
exports.Contract = Contract;
