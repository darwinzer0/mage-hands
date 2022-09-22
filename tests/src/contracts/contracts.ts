import { SecretNetworkClient, Coin, Tx, } from "secretjs";
import { Snip20ContractInstance } from "./snip20";
import { Snip721ContractInstance } from "./snip721";

import { SimulateResponse } from "secretjs/dist/protobuf_stuff/cosmos/tx/v1beta1/service";

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
    contract: ContractInfo;

    constructor(name: string, contract: ContractInfo) {
        this.name = name;
        this.contract = contract;
        this.address = null;
    }

    isInstantiated(): boolean {
        return this.address !== null;
    }

    async instantiate(secretjs: SecretNetworkClient, initMsg: object, label: string, gasLimit: number): Promise<string> {
        if (this.isInstantiated()) {
            throw new Error("Contract is already instantiated");
        }
        const tx = await secretjs.tx.compute.instantiateContract(
            {
                sender: secretjs.address,
                codeId: this.contract.codeId,
                codeHash: this.contract.codeHash,
                initMsg,
                label,
                initFunds: [],
            },
            {
                gasLimit,
            },
        );
        console.dir(tx, {depth: null});
        this.address = tx.arrayLog.find((log) => log.type === "message" && log.key === "contract_address").value;
        return this.address;
    }

    async exec(secretjs: SecretNetworkClient, msg: object, gasLimit: number, sentFunds: Coin[] = []): Promise<Tx> {
        return await secretjs.tx.compute.executeContract(
            {
                sender: secretjs.address,
                contractAddress: this.address,
                codeHash: this.contract.codeHash,
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
                codeHash: this.contract.codeHash,
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
            codeHash: this.contract.codeHash,
            query,
        });
    }
}

export interface Contracts {
    [key: string]: Snip20ContractInstance | 
                   Snip721ContractInstance,
}