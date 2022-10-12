import { keplrStore } from "../stores/keplr";
import { get } from 'svelte/store';

// Pulsar-2
//export const CONTRACT = 'secret1jz687dpv3lwe7pxw8vljhtkc9ysx4a6exrakfx';

// Local testnet
export const PLATFORM_CONTRACT = 'secret1qxxlalvsdjd07p07y3rc5fu6ll8k4tme6e2scc';
export const PLATFORM_CODE_HASH = 'TODO';
export const SNIP20_CONTRACT = 'secretxxx';
export const SNIP20_CODE_HASH = 'TODO';

const utf8decoder = new TextDecoder();

interface Response {
    status: string;
    msg: string;
    key?: string;
};

export interface CreateResponse {
    create?: Response;
    error?: string;
};

export interface ContributeResponse {
    contribute?: Response;
    error?: string;
};

export interface RefundResponse {
    refund?: Response;
    error?: string;
};

export interface PayOutResponse {
    pay_out?: Response;
    error?: string;
};

export interface ContractInfo {
    code_hash: string,
    address: string,
}

interface InnerProjectsResponse {
    projects: ContractInfo[];
    count: number;
};

export interface ProjectsResponse {
    projects?: InnerProjectsResponse;
    error?: string,
};

export interface ProjectStatus {
    id: string;
    creator: string;       
    status: string;
    paid_out: boolean;
    goal: number;
    total: number;
    deadline: Date;
    title: string;
    subtitle: string;
    description: string;
    categories: number[];
    pledged_message?: string;
    funded_message?: string;
    contribution?: number;
};

export interface ProjectStatusResponse {
    projectStatus?: ProjectStatus;
    error?: string;
};

const entropyGenerator = (length): string => {
	var base = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'.split('');
	var array = new Uint8Array(length);
	window.crypto.getRandomValues(array);
	var str = '';
	for (var i = 0; i < array.length; i++) {
		str += base[array[i]%base.length];
	};
	return str;
}

///
/// Execute
///

/*

export const executeContribute = async (
    contract: ContractInfo,
    contribution: string,
    anonymous?: boolean,
): Promise<ContributeResponse> => {
    const keplr = get(keplrStore);
    const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;
    if (!keplrEnabled || !scrtAuthorized) {
        return { error: "Keplr not enabled" };
    }

    const contributionUscrt = (Math.floor(parseFloat(contribution) * 1000000)).toString();
    const entropy = entropyGenerator(32);

    const executeMsg = { 
        contribute: { anonymous: Boolean(anonymous), entropy }
    };

    const funds = [{ denom: "uscrt", amount: contributionUscrt }];
    const fee = {
        amount: [{ amount: "12500", denom: "uscrt" }],
        gas: "50000",
    };

    try {
        const response = await scrtClient.execute(contract.address, executeMsg, "", funds, fee, contract.code_hash);
        let data: ContributeResponse = JSON.parse(utf8decoder.decode(response.data));
        return data;
    } catch (error) {
        return { error: error.toString() };
    }
    
}

export const executeRefund = async (
    contract: ContractInfo,
): Promise<RefundResponse> => {
    const keplr = get(keplrStore);
    const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;
    if (!keplrEnabled || !scrtAuthorized) {
        return { error: "Keplr not enabled" };
    }

    const executeMsg = { 
        refund: { }
    };

    const fee = {
        amount: [{ amount: "12500", denom: "uscrt" }],
        gas: "50000",
    };

    try {
        const response = await scrtClient.execute(contract.address, executeMsg, "", [], fee, contract.code_hash);
        let data: RefundResponse = JSON.parse(utf8decoder.decode(response.data));
        return data;
    } catch (error) {
        return { error: error.toString() };
    }
    
}

export const executePayOut = async (
    contract: ContractInfo,
): Promise<PayOutResponse> => {
    const keplr = get(keplrStore);
    const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;
    if (!keplrEnabled || !scrtAuthorized) {
        return { error: "Keplr not enabled" };
    }

    const executeMsg = { 
        pay_out: { }
    };

    const fee = {
        amount: [{ amount: "12500", denom: "uscrt" }],
        gas: "50000",
    };

    try {
        const response = await scrtClient.execute(contract.address, executeMsg, "", [], fee, contract.code_hash);
        let data: PayOutResponse = JSON.parse(utf8decoder.decode(response.data));
        return data;
    } catch (error) {
        return { error: error.toString() };
    }
    
}
*/