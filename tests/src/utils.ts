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