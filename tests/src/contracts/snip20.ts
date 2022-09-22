import { SecretNetworkClient, fromUtf8, Permit, Tx} from "secretjs";
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
    async instantiate(secretjs: SecretNetworkClient, initMsg: Snip20InitMsg, label: string): Promise<string> {
        return super.instantiate(secretjs, initMsg, label, 100_000);
    }

    async deposit(secretjs: SecretNetworkClient, amount: string): Promise<Tx> {
        const msg = { deposit: { padding: ":::::::::::::::::" } };
        const tx = await this.exec(secretjs, msg, 40_000, [{amount, denom: "uscrt"}]);
        return tx;
    }

    async send(secretjs: SecretNetworkClient, recipient: string, amount: string, gasLimit: number = 100_000): Promise<Tx> {
        const msg = { send: { recipient, amount, padding: ":::::::::::::::::" } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async createViewingKey(secretjs: SecretNetworkClient): Promise<string> {
        const msg = { create_viewing_key: { entropy: entropy(), padding: ":::::::::::::::::" } };
        const { data } = await this.exec(secretjs, msg, 30_000);
        return JSON.parse(fromUtf8(data[0])).create_viewing_key.key;
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