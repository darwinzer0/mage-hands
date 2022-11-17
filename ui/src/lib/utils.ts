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

export const timeUntilDeadline = (currentBlock: number, deadline: number): string => {
    if (deadline < currentBlock) {
        const diff = currentBlock - deadline;
        if (diff > daysInBlocks(1)) {
            const days = Math.floor(diff / (24 * 60 * 10));
            return days === 1 ? `${days} day ago` : `${days} days ago`;
        } else if (diff > hoursInBlocks(1)) {
            const hours = Math.floor(diff / (60 * 10));
            return hours === 1 ? `${hours} hour ago` : `${hours} hours ago`;
        } else if (diff > minutesInBlocks(1)) {
            const minutes = Math.floor(diff / 10);
            return minutes === 1 ? `${minutes} minute ago` : `${minutes} minutes ago`;
        } else {
            return `less than one minute ago`;
        }
    } else {
        const diff = deadline - currentBlock;
        if (diff > daysInBlocks(1)) {
            const days = Math.ceil(diff / (24 * 60 * 10));
            return `less than ${days} days left`;
        } else if (diff > hoursInBlocks(1)) {
            const hours = Math.ceil(diff / (60 * 10));
            return `less than ${hours} hours left`;
        } else if (diff > minutesInBlocks(1)) {
            const minutes = Math.ceil(diff / 10);
            return `less than ${minutes} minutes left`;
        } else {
            return `less than one minute left`;
        }
    }
}