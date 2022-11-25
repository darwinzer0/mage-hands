import { SecretNetworkClient, Tx} from "secretjs";
import { ContractInstance } from "./contracts";
import { ProjectRewardMessage, Snip24RewardInit } from "./project";

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
    cover_img: string;
    pledged_message?: string;
    funded_message?: string;
    reward_messages: ProjectRewardMessage[];
    goal: string;
    deadline: number;
    categories: number[];
    snip20_contract: string;
    snip20_hash: string;
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
    projects: {
        projects: PlatformContractInfo[];
        count: number;
    };
}

export class PlatformContractInstance extends ContractInstance {

    async create(secretjs: SecretNetworkClient, createMsg: PlatformCreateMsg, gasLimit: number = 3_000_000): Promise<Tx> {
        const msg = { create: createMsg };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async config(secretjs: SecretNetworkClient, configMsg: PlatformConfigMsg, gasLimit: number = 250_000): Promise<Tx> {
        const msg = { config: configMsg };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async register(secretjs: SecretNetworkClient, registerMsg: PlatformRegisterMsg, gasLimit: number = 250_000): Promise<Tx> {
        const msg = { register: registerMsg };
        const tx = await this.exec(secretjs, msg, gasLimit);
        return tx;
    }

    async queryProjects(secretjs: SecretNetworkClient, page: number = 0, page_size: number = 10): Promise<PlatformProjectsResult> {
        const query = { projects: { page, page_size } };
        const result = (await this.query(secretjs, query)) as PlatformProjectsResult;
        return result;
    }

}
