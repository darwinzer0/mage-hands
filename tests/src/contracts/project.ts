import { SecretNetworkClient, fromUtf8, Permit, Tx} from "secretjs";
import { entropy } from "../utils";
import { ContractInstance } from "./contracts";

export type ProjectInitMsg = {
    creator: string;
    title: string;
    subtitle?: string;
    description: string;
    pledged_message?: string;
    funded_message?: string;
    reward_messages: RewardMessage[];
    goal: string;
    deadline: number;
    deadman: number;
    categories: number[];
    entropy: string;
    source_contract: string;
    source_hash: string;
    snip20_contract: string;
    snip20_hash: string;
    minimum_pledge: string;
    maximum_pledge: string;
    padding?: string;
}

export type Snip24RewardInit = {
    reward_snip24_code_id: number;
    reward_snip24_code_hash: string;
    name: string;
    admin: string;
    symbol: string;
    decimals: number;
    public_total_supply: boolean;
    enable_deposit: boolean;
    enable_redeem: boolean;
    enable_mint: boolean;
    enable_burn: boolean;
    amount: string;
    contributors_vesting_schedule: VestingEvent[];
    contributors_per_mille: number;
    minimum_contribution: string;
    contribution_weight: number;
    creator_vesting_schedule: VestingEvent[];
    creator_per_mille: number;
    creator_addresses?: string[];
}

export type VestingEvent = {
    block: number;
    per_mille: number;
}

export type ProjectChangeTextMsg = {
    title?: string;
    subtitle?: string;
    description?: string;
    pledged_message?: string;
    funded_message?: string;
    categories?: number[];
    padding?: string;
}

export type ProjectStatusResult = {
    creator: string;
    status: string;
    paid_out: boolean;
    goal: string;
    total: string;
    deadline: number;
    deadman: number;
    title: string;
    subtitle: string;
    description: string;
    categories: number[];
    spam_count: number;
    snip20_address: string,
    minimum_pledge: string,
    maximum_pledge: string,
    pledged_message?: string;
    funded_message?: string;
    reward_messages?: RewardMessage[];
    contribution?: string;
}

export type RewardMessage = {
    threshold: string;
    message: string;
}

export class ProjectContractInstance extends ContractInstance {
    async instantiate(secretjs: SecretNetworkClient, initMsg: ProjectInitMsg, label: string, gasLimit: number = 150_000): Promise<string> {
        return super.instantiate(secretjs, initMsg, label, gasLimit);
    }

    async changeText(secretjs: SecretNetworkClient, changeTextMsg: ProjectChangeTextMsg, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { change_text: changeTextMsg };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async refund(secretjs: SecretNetworkClient, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { refund: { padding: "=========" } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async cancel(secretjs: SecretNetworkClient, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { cancel: { padding: "=========" } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async comment(secretjs: SecretNetworkClient, comment: string, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { comment: { comment, padding: "=========" }};
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async flag_spam(secretjs: SecretNetworkClient, flag: boolean, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { flag_spam: { flag, padding: "=========" }};
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async payOut(secretjs: SecretNetworkClient, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { pay_out: { padding: "=========" } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async generateViewingKey(secretjs: SecretNetworkClient, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { generate_viewing_key: { entropy: entropy(), padding: "=========" } };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async queryStatus(secretjs: SecretNetworkClient): Promise<ProjectStatusResult> {
        const query = { status: { } };
        const result = (await this.query(secretjs, query)) as ProjectStatusResult;
        return result;
    }

    async queryStatusPermit(secretjs: SecretNetworkClient, permit: Permit): Promise<ProjectStatusResult> {
        const query = { status_with_permit: { permit } };
        const result = (await this.query(secretjs, query)) as ProjectStatusResult;
        return result;
    }

}