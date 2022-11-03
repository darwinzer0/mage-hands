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
