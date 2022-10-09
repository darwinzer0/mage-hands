import { SecretNetworkClient, fromUtf8, Permit, Tx} from "secretjs";
import { entropy } from "../utils";
import { ContractInstance } from "./contracts";
import { RewardMessage, Snip24RewardInit } from "./project";

export type PlatformInitMsg = {
    owner?: string;
    project_contract_code_id: number;
    project_contract_code_hash: string;
    token_min_max_pledges: PledgeMinMax[];
    deadman?: number;
}

export type PledgeMinMax = {
    token_addr: string;
    min: string;
    max: string;
}

export type PlatformCreateMsg = {
    title: string;
    subtitle?: string;
    description: string;
    pledged_message?: string;
    funded_message?: string;
    reward_messages: RewardMessage[];
    goal: string;
    deadline: number;
    categories: number[];
    snip24_reward_init?: Snip24RewardInit;
    entropy: string; // used to set up prng in project contract
    padding?: string;
}

export type PlatformConfigMsg = {
    owner?: string;
    project_contract_code_id?: number;
    project_contract_code_hash?: string;
    deadman?: number;
    padding?: string;
}

export type PlatformRegisterMsg = {
    contract_addr: string;
    contract_code_hash: string;
}

export type PlatformContractInfo = {
    code_hash: string;
    address: string;
}

export type PlatformProjectsResult = {
    projects: PlatformContractInfo[];
    count: number;
}

export class PlatformContractInstance extends ContractInstance {
    async instantiate(secretjs: SecretNetworkClient, initMsg: PlatformInitMsg, label: string, gasLimit: number = 150_000): Promise<string> {
        return super.instantiate(secretjs, initMsg, label, gasLimit);
    }

    async create(secretjs: SecretNetworkClient, createMsg: PlatformCreateMsg, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { create: createMsg };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async config(secretjs: SecretNetworkClient, configMsg: PlatformConfigMsg, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { config: configMsg };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async register(secretjs: SecretNetworkClient, registerMsg: PlatformRegisterMsg, gasLimit: number = 150_000): Promise<Tx> {
        const msg = { register: registerMsg };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async queryProjects(secretjs: SecretNetworkClient, page: number, page_size: number): Promise<PlatformProjectsResult> {
        const query = { projects: { page, page_size } };
        const result = (await this.query(secretjs, query)) as PlatformProjectsResult;
        return result;
    }

}
