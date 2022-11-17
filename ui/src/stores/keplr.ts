import { writable } from "svelte/store";
import type { ChainInfo, Window as KeplrWindow } from "@keplr-wallet/types";
import { SecretNetworkClient, StdSignature } from 'secretjs';
import { permitName } from "./permits";
import { PLATFORM_CONTRACT } from "src/lib/env";

// Pulsar
//export const CHAIN_ID = 'pulsar-2';
//export const SECRET_grpcWebUrl = 'https://grpc.testnet.secretsaturn.net';
//export const SECRET_LCD = 'https://lcd.testnet.secretsaturn.net';

// Local testnet
export const CHAIN_ID = 'secretdev-1';
export const SECRET_grpcWebUrl = "http://localhost:9091";
export const SECRET_LCD = "http://localhost:1317";

declare global {
    // eslint-disable-next-line @typescript-eslint/no-empty-interface
    interface Window extends KeplrWindow {}
}

export interface KeplrStore {
    keplrEnabled: boolean;
    scrtAuthorized: boolean;
    scrtClient?: SecretNetworkClient;
};

function createKeplrStore() {
    let keplrStoreNew: KeplrStore = {
        keplrEnabled: false,
        scrtAuthorized: false,
        scrtClient: null,
    };

	const { subscribe, set, update } = writable(keplrStoreNew);

	return {
		subscribe,
		connect: async () => {
            console.log("connect to keplr");
            const keplr = await connectKeplr(CHAIN_ID, SECRET_grpcWebUrl);
            set(keplr);
        },
	};
}

async function checkKeplr(chainId: string) {
    let keplrEnabled = false;
    const keplrCheckPromise = new Promise<void> ( (resolve, reject) => {
        const keplrCheckInterval = setInterval(async () => {
            let isKeplrWallet = !!window.keplr && !!window.getOfflineSigner && !!window.getEnigmaUtils;
            if (isKeplrWallet) {
                keplrEnabled = true;
                clearInterval(keplrCheckInterval);

                if (chainId === 'secretdev-1' || chainId === 'pulsar-2') {
                    await suggestChain(chainId);
                }
                resolve();
            }
        }, 1000);
    });
    await keplrCheckPromise;
    return keplrEnabled;
}

async function connectKeplr(chainId: string, grpcWebUrl: string) {
    let keplrEnabled = await checkKeplr(chainId);
    let scrtAuthorized = false;
    let scrtClient: SecretNetworkClient = null;
    try {
        await window.keplr.enable(chainId);
        const keplrOfflineSigner = window.keplr.getOfflineSignerOnlyAmino(chainId);
        const [{ address: myAddress }] = await keplrOfflineSigner.getAccounts();

        const secretjs = await SecretNetworkClient.create({
            grpcWebUrl,
            chainId: CHAIN_ID,
            wallet: keplrOfflineSigner,
            walletAddress: myAddress,
            encryptionUtils: window.keplr.getEnigmaUtils(chainId),
        });
        scrtAuthorized = true;
        scrtClient = secretjs;
    } catch (error) {
        scrtAuthorized = false;
        scrtClient = null;
    }
    let keplr: KeplrStore = {
        keplrEnabled,
        scrtAuthorized,
        scrtClient,
    };
    window.addEventListener("keplr_keystorechange", () => {
        window.location.reload();
    });
    return keplr;
}

async function suggestChain(chainId) {
    let rpc, rest, chainName;

    if (chainId === 'secretdev-1') {
        rpc = "http://localhost:26657/";
        rest = SECRET_LCD;
        chainName = "Localhost Secret Testnet";
    }
    //} else if (chainId === 'holodeck-2') {
    //    rpc = "https://chainofsecrets.secrettestnet.io:26667/";
    //    rest = SECRET_LCD;
    //    chainName = "Supernova-2 Secret Testnet";
    //}

    let newChain: ChainInfo = {
        chainId: chainId,
        bip44: {
            coinType: 529,
        },
        coinType: 529,
        stakeCurrency: {
            coinDenom: 'SCRT',
            coinMinimalDenom: 'uscrt',
            coinDecimals: 6,
        },
        bech32Config: {
            bech32PrefixAccAddr: 'secret',
            bech32PrefixAccPub: 'secretpub',
            bech32PrefixValAddr: 'secretvaloper',
            bech32PrefixValPub: 'secretvaloperpub',
            bech32PrefixConsAddr: 'secretvalcons',
            bech32PrefixConsPub: 'secretvalconspub',
        },
        currencies: [
            {
                coinDenom: 'SCRT',
                coinMinimalDenom: 'uscrt',
                coinDecimals: 6,
            },
        ],
        feeCurrencies: [
            {
                coinDenom: 'SCRT',
                coinMinimalDenom: 'uscrt',
                coinDecimals: 6,
            },
        ],
        gasPriceStep: {
            low: 0.1,
            average: 0.25,
            high: 0.4,
        },
        features: ['secretwasm'],
        rpc,
        rest,
        chainName
    };
  
    if (newChain.rpc && newChain.rest && window.keplr) {
        await window.keplr.experimentalSuggestChain(newChain);
    }
}

export async function getSignature(chainId): Promise<StdSignature> {
    const keplrOfflineSigner = window.getOfflineSigner(chainId);
    const accounts = await keplrOfflineSigner.getAccounts();
    const myAddress = accounts[0].address;

    const { signature } = await window.keplr.signAmino(
        chainId,
        myAddress,
        {
            chain_id: chainId,
            account_number: "0",
            sequence: "0",
            fee: {
                amount: [{ denom: "uscrt", amount: "0" }],
                gas: "1",
            },
            msgs: [
                {
                    type: "query_permit",
                    value: {
                        permit_name: permitName,
                        allowed_tokens: [PLATFORM_CONTRACT],
                        permissions: ["owner"],
                    },
                },
            ],
            memo: "",
        },
        {
            preferNoSetFee: true,
            preferNoSetMemo: true,
        }
    );
    return signature;
}

export const keplrStore = createKeplrStore();
