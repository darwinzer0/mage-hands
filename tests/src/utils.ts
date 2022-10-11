import { randomBytes } from "crypto";
import { toUtf8, toBase64, } from "secretjs";

export const entropy = (): string => {
    return randomBytes(32).toString('hex');
}

export const sleep = (ms: number): Promise<void> => {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

export const banner = (str: string) => {
    console.log("================================");
    console.log(str);
    console.log("================================\n");
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

export const p = (s: any) => {
    console.dir(s, {depth: null});
}