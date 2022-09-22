import { Wallet, SecretNetworkClient, newPermit, Permit, fromUtf8, toBase64} from "secretjs";
import { readFileSync } from "fs";
import { a, b, c, d } from "./accounts";
import { 
    contracts, 
    ContractInfo, 
    uploadContract,
    Snip20ContractInstance,
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
    contracts.snip20 = new Snip20ContractInstance("sSCRT", sscrtCode);
    const { snip20 } = contracts;
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
    await snip20.instantiate(secretjs, sscrtInitMsg, `sSCRT-${sscrtCode.codeId}`);
    console.log(snip20.address);

    console.log("Seeding a, b, c, d with 1,000,000 sSCRT");
    await snip20.deposit(a.signer, "1000000000000");
    await snip20.deposit(b.signer, "1000000000000");
    await snip20.deposit(c.signer, "1000000000000");
    await snip20.deposit(d.signer, "1000000000000");

    console.log("Creating sSCRT viewing keys for a, b, c, d")
    a.sScrt.viewingKey = await snip20.createViewingKey(a.signer);
    b.sScrt.viewingKey = await snip20.createViewingKey(b.signer);
    c.sScrt.viewingKey = await snip20.createViewingKey(c.signer);
    d.sScrt.viewingKey = await snip20.createViewingKey(d.signer);

    console.log("Creating sSCRT query permits for a, b, c, d");
    a.sScrt.queryPermit = await newPermit(a.wallet, a.wallet.address, CHAIN_ID, "test", [snip20.address], ["owner", "balance"], false);
    console.log(a.sScrt.queryPermit);
    b.sScrt.queryPermit = await newPermit(b.wallet, b.wallet.address, CHAIN_ID, "test", [snip20.address], ["owner", "balance"], false);
    console.log(b.sScrt.queryPermit);
    c.sScrt.queryPermit = await newPermit(c.wallet, c.wallet.address, CHAIN_ID, "test", [snip20.address], ["owner", "balance"], false);
    console.log(c.sScrt.queryPermit);
    d.sScrt.queryPermit = await newPermit(d.wallet, d.wallet.address, CHAIN_ID, "test", [snip20.address], ["owner", "balance"], false);
    console.log(d.sScrt.queryPermit);

    await printAllSnip20Balances(snip20);
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

    console.log("DONE");
}

(async () => {
    main();
})();