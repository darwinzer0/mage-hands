import { Wallet, SecretNetworkClient, newPermit, Permit, fromUtf8, toBase64} from "secretjs";
import { readFileSync } from "fs";
import { a, b, c, d } from "./accounts";
import { 
    contracts, 
    ContractInfo, 
    uploadContract,
    Snip20ContractInstance,
    PlatformContractInstance,
} from "./contracts";
import { banner, base64, entropy, sleep } from "./utils";

const CHAIN_ID = "secretdev-1";

const printBalance = async (name: string, secretjs: SecretNetworkClient) => {
    const { balance: { amount }, } = await secretjs.query.bank.balance({
        address: secretjs.address,
        denom: "uscrt",
    });
      
    console.log(`${name} - ${secretjs.address} has ${Number(amount) / 1e6} SCRT!`);
}

const printAllSnip20Balances = async (snip20: Snip20ContractInstance) => {
    console.log("a sSCRT balance:", await snip20.queryBalance(a.signer, a.sScrt.queryPermit));
    console.log("b sSCRT balance:", await snip20.queryBalance(b.signer, b.sScrt.queryPermit));
    console.log("c sSCRT balance:", await snip20.queryBalance(c.signer, c.sScrt.queryPermit));
    console.log("d sSCRT balance:", await snip20.queryBalance(d.signer, d.sScrt.queryPermit));
}

//
// Uploads the sSCRT SNIP-20 contract and initializes.
// It also deposits 1,000,000 SCRT for a, b, c, d, and creates viewing keys and query permits for each account
// Used during set up of local dev testnet
// Puts contract info in the `snip20` field of the global `contracts` object
//
const setupSScrtContract = async (secretjs: SecretNetworkClient) => {
    banner("Setting up sSCRT contract (SNIP-20)");
    
    /// Upload and initialize the sSCRT contract code
    console.log("Uploading sSCRT contract code");
    const sscrtCode: ContractInfo = await uploadContract(
        secretjs, 
        readFileSync(`${__dirname}/../../snip20-reference-impl/contract.wasm.gz`) as Uint8Array,
        "",
        "", 
        3_000_000
    );
    console.log(sscrtCode.codeId, sscrtCode.codeHash);

    console.log("Instantiating sSCRT contract");
    contracts.sscrt = new Snip20ContractInstance("sSCRT", sscrtCode);
    const { sscrt } = contracts;
    const sscrtInitMsg = {
        name: "Secret SCRT",
        admin: a.signer.address,
        symbol: "SSCRT",
        decimals: 6,
        initial_balances: [],
        prng_seed: Buffer.from(entropy(), 'utf8').toString('base64'),
        config: {
            public_total_supply: true,
            enable_deposit: true,
            enable_redeem: true,
            enable_mint: false,
            enable_burn: false,
        },
        supported_denoms: ["uscrt"],
    };
    await sscrt.instantiate(secretjs, sscrtInitMsg, `sSCRT-${sscrtCode.codeId}`);
    console.log(sscrt.address);

    console.log("Seeding a, b, c, d with 1,000,000 sSCRT");
    await sscrt.deposit(a.signer, "1000000000000");
    await sscrt.deposit(b.signer, "1000000000000");
    await sscrt.deposit(c.signer, "1000000000000");
    await sscrt.deposit(d.signer, "1000000000000");

    console.log("Creating sSCRT viewing keys for a, b, c, d")
    a.sScrt.viewingKey = await sscrt.createViewingKey(a.signer);
    b.sScrt.viewingKey = await sscrt.createViewingKey(b.signer);
    c.sScrt.viewingKey = await sscrt.createViewingKey(c.signer);
    d.sScrt.viewingKey = await sscrt.createViewingKey(d.signer);

    console.log("Creating sSCRT query permits for a, b, c, d");
    a.sScrt.queryPermit = await newPermit(a.wallet, a.wallet.address, CHAIN_ID, "test", [sscrt.address], ["owner", "balance"], false);
    console.log(a.sScrt.queryPermit);
    b.sScrt.queryPermit = await newPermit(b.wallet, b.wallet.address, CHAIN_ID, "test", [sscrt.address], ["owner", "balance"], false);
    console.log(b.sScrt.queryPermit);
    c.sScrt.queryPermit = await newPermit(c.wallet, c.wallet.address, CHAIN_ID, "test", [sscrt.address], ["owner", "balance"], false);
    console.log(c.sScrt.queryPermit);
    d.sScrt.queryPermit = await newPermit(d.wallet, d.wallet.address, CHAIN_ID, "test", [sscrt.address], ["owner", "balance"], false);
    console.log(d.sScrt.queryPermit);

    await printAllSnip20Balances(sscrt);
}

//
// Uploads the project contract
//
const setupProjectContract = async (secretjs: SecretNetworkClient): Promise<ContractInfo> => {
    banner("Setting up project contract");

    /// Upload the project contract code
    console.log("Uploading project contract code");
    const projectCode: ContractInfo = await uploadContract(
        secretjs, 
        readFileSync(`${__dirname}/../../contracts/mage-hands-project/contract.wasm.gz`) as Uint8Array,
        "",
        "", 
        4_000_000
    );
    console.log(projectCode.codeId, projectCode.codeHash);

    return projectCode;
}

//
// Uploads the platform contract and initializes.
//
const setupPlatformContract = async (
    secretjs: SecretNetworkClient, 
    projectContractInfo: ContractInfo
) => {
    banner("Setting up platform contract");
    
    /// Upload and initialize the platform contract code
    console.log("Uploading platform contract code");
    const platformCode: ContractInfo = await uploadContract(
        secretjs, 
        readFileSync(`${__dirname}/../../contracts/mage-hands-platform/contract.wasm.gz`) as Uint8Array,
        "",
        "", 
        4_000_000
    );
    console.log(platformCode.codeId, platformCode.codeHash);

    console.log("Instantiating platform contract");
    contracts.platform = new PlatformContractInstance("platform", platformCode);
    const { sscrt, platform } = contracts;
    const platformInitMsg = {
        project_contract_code_id: projectContractInfo.codeId,
        project_contract_code_hash: projectContractInfo.codeHash,
        token_min_max_pledges: [
            {
                token_addr: sscrt.address,
                min: "1000000", // 1 sscrt
                max: "10000000000", // 10000 sscrt
            },
        ],
    };
    await platform.instantiate(secretjs, platformInitMsg, `platform-${platformCode.codeId}`);
    console.log(platform.address);
}

const main = async () => {
    console.log("Creating signers for a, b, c, d");
    a.signer = await SecretNetworkClient.create({
        grpcWebUrl: "http://localhost:9091",
        chainId: CHAIN_ID,
        wallet: a.wallet,
        walletAddress: a.wallet.address,
    });
    printBalance('a', a.signer);

    b.signer = await SecretNetworkClient.create({
        grpcWebUrl: "http://localhost:9091",
        chainId: CHAIN_ID,
        wallet: b.wallet,
        walletAddress: b.wallet.address,
    });
    printBalance('b', b.signer);

    c.signer = await SecretNetworkClient.create({
        grpcWebUrl: "http://localhost:9091",
        chainId: CHAIN_ID,
        wallet: c.wallet,
        walletAddress: c.wallet.address,
    });
    printBalance('c', c.signer);

    d.signer = await SecretNetworkClient.create({
        grpcWebUrl: "http://localhost:9091",
        chainId: CHAIN_ID,
        wallet: d.wallet,
        walletAddress: d.wallet.address,
    });
    printBalance('d', d.signer);

    banner("Uploading all contract wasm code");

    // sets up the sSCRT/SNIP-20 contract for dev testnet
    // comment out if running on chain already with sSCRT, and manually add existing contract info to contracts.snip20
    // TODO: add default values for contracts.snip20 on pulsar and mainnet
    await setupSScrtContract(a.signer);

    let projectContractInfo = await setupProjectContract(a.signer);
    await setupPlatformContract(a.signer, projectContractInfo);

    console.log("DONE");
}

(async () => {
    main();
})();