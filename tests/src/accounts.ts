import { Wallet, SecretNetworkClient, newPermit, Permit, fromUtf8} from "secretjs";

export interface Account {
    wallet: Wallet,
    signer?: SecretNetworkClient,
    sScrt: {
        viewingKey?: string,
        queryPermit?: Permit,
    },
    platform: {
        viewingKey?: string,
        queryPermit?: Permit,
    },
}

// default accounts in LocalSecret
export const a: Account = {
    wallet: new Wallet('grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar'),
    sScrt: {},
    platform: {},
};

export const b: Account = { 
    wallet: new Wallet('jelly shadow frog dirt dragon use armed praise universe win jungle close inmate rain oil canvas beauty pioneer chef soccer icon dizzy thunder meadow'),
    sScrt: {},
    platform: {},
};

export const c: Account = { 
    wallet: new Wallet('chair love bleak wonder skirt permit say assist aunt credit roast size obtain minute throw sand usual age smart exact enough room shadow charge'),
    sScrt: {},
    platform: {},
};

export const d: Account = {
    wallet: new Wallet('word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick'),
    sScrt: {},
    platform: {},
};

