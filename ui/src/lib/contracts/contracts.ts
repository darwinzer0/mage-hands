import { SecretNetworkClient, Coin, Tx, } from "secretjs";
import { Snip20ContractInstance } from "./snip20";
import { Snip721ContractInstance } from "./snip721";

import { SimulateResponse } from "secretjs/dist/protobuf_stuff/cosmos/tx/v1beta1/service";
import { PlatformContractInstance } from "./platform";
import { ProjectContractInstance } from "./project";

export type ContractInfo = {
    codeHash: string;
    codeId: number;
}

export const uploadContract = async(
    secretjs: SecretNetworkClient, 
    wasmByteCode: Uint8Array, 
    source: string, 
    builder: string, 
    gasLimit: number
): Promise<ContractInfo> => {
    const uploadTx = await secretjs.tx.compute.storeCode(
        {
            sender: secretjs.address,
            wasmByteCode,
            source,
            builder,
        },
        {
            gasLimit,
        },
    );
    const codeId = Number(
        uploadTx.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
          .value,
    );
    const codeHash = await secretjs.query.compute.codeHash(codeId);
    return {codeId, codeHash};
}

export class ContractInstance {
    name: string;
    address: string;
    codeHash: string;

    constructor(name: string, codeHash: string = null, address: string = null) {
        this.name = name;
        this.codeHash = codeHash;
        this.address = address;
    }

    isInstantiated(): boolean {
        return this.address !== null;
    }

    async exec(secretjs: SecretNetworkClient, msg: object, gasLimit: number, sentFunds: Coin[] = []): Promise<Tx> {
        return await secretjs.tx.compute.executeContract(
            {
                sender: secretjs.address,
                contractAddress: this.address,
                codeHash: this.codeHash,
                msg,
                sentFunds,
            },
            {
                gasLimit,
            },
        );
    }

    async simulate(secretjs: SecretNetworkClient, msg: object, gasLimit: number, sentFunds: Coin[] = []): Promise<SimulateResponse> {
        return await secretjs.tx.compute.executeContract.simulate(
            {
                sender: secretjs.address,
                contractAddress: this.address,
                codeHash: this.codeHash,
                msg,
                sentFunds,
            },
            {
                gasLimit,
            },
        )
    }

    async query(secretjs: SecretNetworkClient, query: object): Promise<object> {
        return await secretjs.query.compute.queryContract({
            contractAddress: this.address,
            codeHash: this.codeHash,
            query,
        });
    }
}

export interface Contracts {
    [key: string]: Snip20ContractInstance | 
                   Snip721ContractInstance |
                   PlatformContractInstance |
                   ProjectContractInstance
}