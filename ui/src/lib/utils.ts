import { randomBytes } from "crypto";
import { toUtf8, toBase64, SecretNetworkClient, } from "secretjs";

export const entropy = (): string => {
    return randomBytes(32).toString('hex');
}

export const base64 = (str: string): string => {
    return toBase64(toUtf8(str));
}

// assumes 1 block / 6 sec
export const minutesInBlocks = (minutes: number): number => {
    return minutes * 10;
}

export const hoursInBlocks = (hours: number): number => {
    return minutesInBlocks(hours * 60);
}

export const daysInBlocks = (days: number): number => {
    return hoursInBlocks(days * 24);
}

export const getBlock = async (scrtClient: SecretNetworkClient) : Promise<number> => {
    const latestBlockResponse = await scrtClient.query.tendermint.getLatestBlock({});
    const block = parseInt(latestBlockResponse.block.header.height);
    return block;
}