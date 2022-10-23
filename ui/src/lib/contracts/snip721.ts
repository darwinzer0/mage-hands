import { SecretNetworkClient, fromUtf8, Permit, Coin, Tx, } from "secretjs";
import { ContractInstance } from "./contracts";

export type Snip721TokensResult = {
    tokens: {
        token_list: string[],
    };
};

export type Snip721Trait = {
    display_type?: string,
    trait_type?: string,
    value: string,
    max_value?: string,
};

export type Snip721Authentication = {
    key?: string,
    user?: string,
};

export type Snip721MediaFile = {
    file_type?: string,
    extension?: string,
    authentication?: Snip721Authentication,
    url?: string,
};

export type Snip721Extension = {
    image?: string,
    image_data?: string,
    external_url?: string,
    description?: string,
    name?: string,
    attributes?: Snip721Trait[],
    background_color?: string,
    animation_url?: string,
    youtube_url?: string,
    media?: Snip721MediaFile[],
    protected_attributes?: string[],
}

export type Snip721Metadata = {
    token_uri?: string,
    extension?: Snip721Extension,
};

export type Snip721DisplayRoyalty = {
    recipient?: string,
    rate: number,
};

export type Snip721DisplayRoyaltyInfo = {
    decimal_places_in_rates: number,
    royalties: Snip721DisplayRoyalty[],
};

export type Snip721MintRunInfo = {
    collection_creator?: string,
    token_creator?: string,
    time_of_minting?: number,
    mint_run?: number,
    serial_number?: number,
    quantity_minted_this_run?: number,
};

export type Snip721Expiration = "at_height" | "at_time" | "never";

export type Snip721Approval = {
    address: string;
    view_owner_expiration?: Snip721Expiration,
    view_private_metadata_expiration?: Snip721Expiration,
    transfer_expiration?: Snip721Expiration,
};

export type Snip721NftDossierResult = {
    nft_dossier: {
        owner?: string,
        public_metadata?: Snip721Metadata,
        private_metadata?: Snip721Metadata,
        display_private_metadata_error?: string,
        royalty_info?: Snip721DisplayRoyaltyInfo,
        mint_run_info?: Snip721MintRunInfo,
        owner_is_public: boolean,
        public_ownership_expiration?: Snip721Expiration,
        token_approvals?: Snip721Approval[],
        inventory_approvals?: Snip721Approval[],
    };
}

export type Snip721AccessLevel = "approve_token" | "all" | "revoke_token" | "none";

export type Snip721SetWhitelistedApprovalMsg = {
    set_whitelisted_approval: {
        address: string,
        token_id?: string,
        view_owner?: Snip721AccessLevel,
        view_private_metadata?: Snip721AccessLevel,
        transfer?: Snip721AccessLevel,
        expires?: Snip721Expiration,
        padding?: string,
    };
}

export type Snip721InitMsg = {
    name: string;
    symbol: string;
    admin?: string;
    entropy: string;
    royalty_info?: {
        token_id?: string,
        viewer?: {
            address: string,
            viewing_key: string,
        }
    };
    config?: {
        public_token_supply?: boolean,
        public_owner?: boolean,
        enable_sealed_metadata?: boolean,
        unwrapped_metadata_is_private?: boolean,
        minter_may_update_metadata?: boolean,
        owner_may_update_metadata?: boolean,
        enable_burn?: boolean,
    };
    post_init_callback?: {
        msg: string,
        contract_address: string,
        code_hash: string,
        send: Coin[],
    };
    hive_contract?: {
        address: string,
        code_hash: string,
    };
}

export class Snip721ContractInstance extends ContractInstance {

    async batchSendNft(secretjs: SecretNetworkClient, contract: string, token_ids: string[]): Promise<Tx> {
        const msg = { batch_send_nft: { sends: [ { contract, token_ids } ] } };
        const tx = await this.exec(secretjs, msg, 200_000);
        return tx;
    }

    async setMinters(secretjs: SecretNetworkClient, minters: string[]): Promise<boolean> {
        const msg = { set_minters: { minters } };
        const { data } = await this.exec(secretjs, msg, 100_000);
        return JSON.parse(fromUtf8(data[0])).set_minters.status === "success";
    }

    async setWhitelistedApproval(
        secretjs: SecretNetworkClient, 
        address: string, 
        tokenId?: string, 
        viewOwner?: Snip721AccessLevel,
        viewPrivateMetadata?: Snip721AccessLevel,
        transfer?: Snip721AccessLevel,
        expires?: Snip721Expiration,
    ): Promise<Tx> {
        const msg: Snip721SetWhitelistedApprovalMsg = { set_whitelisted_approval: { address } };
        if (tokenId !== undefined) {
            msg.set_whitelisted_approval.token_id = tokenId;
        }
        if (viewOwner !== undefined) {
            msg.set_whitelisted_approval.view_owner = viewOwner;
        }
        if (viewPrivateMetadata !== undefined) {
            msg.set_whitelisted_approval.view_private_metadata = viewPrivateMetadata;
        }
        if (transfer !== undefined) {
            msg.set_whitelisted_approval.transfer = transfer;
        }
        if (expires !== undefined) {
            msg.set_whitelisted_approval.expires = expires;
        }
        const tx = await this.exec(secretjs, msg, 100_000);
        return tx;
    }

    async queryTokens(secretjs: SecretNetworkClient, permit: Permit): Promise<Snip721TokensResult> {
        const query = { with_permit: { query: { tokens: { owner: secretjs.address } }, permit } };
        const result = (await this.query(secretjs, query)) as Snip721TokensResult;
        return result;
    }

    async queryNftDossier(secretjs: SecretNetworkClient, tokenId: string, permit: Permit): Promise<Snip721NftDossierResult> {
        const query = { with_permit: { query: { nft_dossier: { token_id: tokenId } }, permit } };
        const result = (await this.query(secretjs, query)) as Snip721NftDossierResult;
        return result;
    }

}