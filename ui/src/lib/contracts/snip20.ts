import { SecretNetworkClient, fromUtf8, Permit, Tx, } from "secretjs";
import { MsgExecuteContractResponse } from "secretjs/dist/protobuf_stuff/secret/compute/v1beta1/msg";
import { entropy } from "../utils";
import { ContractInstance } from "./contracts";

export type Snip20InitialBalance = {
    address: string;
    amount: string;
}

export type Snip20InitMsg = {
    name: string;
    admin: string;
    symbol: string;
    decimals: number;
    initial_balances?: Snip20InitialBalance[];
    prng_seed: string;
    config?: {
        public_total_supply: boolean,
        enable_deposit: boolean,
        enable_redeem: boolean,
        enable_mint: boolean,
        enable_burn: boolean,
    };
    supported_denoms: string[];
}

export type Snip20BalanceResult = {
    balance: {
        amount: string,
    };
};

export class Snip20ContractInstance extends ContractInstance {

    async deposit(secretjs: SecretNetworkClient, amount: string, gasLimit: number = 250_000): Promise<Tx> {
        const msg = { deposit: { } };
        const tx = await this.exec(secretjs, msg, gasLimit, [{amount, denom: "uscrt"}]);
        return tx;
    }

    async send(secretjs: SecretNetworkClient, recipient: string, amount: string, gasLimit: number = 400_000): Promise<Tx> {
        const msg = { send: { recipient, amount } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async transfer(secretjs: SecretNetworkClient, recipient: string, amount: string, gasLimit: number = 250_000): Promise<Tx> {
        const msg = { transfer: { recipient, amount } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async createViewingKey(secretjs: SecretNetworkClient, gasLimit: number = 250_000): Promise<string> {
        const msg = { create_viewing_key: { entropy: entropy() } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(tx.data[0]).data)).create_viewing_key.key;
    }

    private async queryBalanceViewingKey(secretjs: SecretNetworkClient, vk: string): Promise<Snip20BalanceResult> {
        const query = { balance: { address: secretjs.address, key: vk } };
        const result = (await this.query(secretjs, query)) as Snip20BalanceResult;
        return result;
    }

    private async queryBalancePermit(secretjs: SecretNetworkClient, permit: Permit): Promise<Snip20BalanceResult> {
        const query = { with_permit: { query: { balance: { } }, permit } };
        const result = (await this.query(secretjs, query)) as Snip20BalanceResult;
        return result;
    }

    async queryBalance(secretjs: SecretNetworkClient, auth: string | Permit): Promise<Snip20BalanceResult> {
        if (typeof auth === "string") { // auth is a viewing key
            return this.queryBalanceViewingKey(secretjs, auth);
        } else { // auth is a permit
            return this.queryBalancePermit(secretjs, auth);
        }
    }
}