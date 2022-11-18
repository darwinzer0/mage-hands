import { SecretNetworkClient, newPermit, Permit, fromUtf8, toBase64} from "secretjs";
import { MsgExecuteContractResponse } from "secretjs/dist/protobuf_stuff/secret/compute/v1beta1/msg";
import { GetLatestBlockResponse } from "secretjs/dist/protobuf_stuff/cosmos/base/tendermint/v1beta1/query";
import { readFileSync } from "fs";
import { a, b, c, d } from "./accounts";
import { 
    contracts, 
    ContractInfo, 
    uploadContract,
    Snip20ContractInstance,
    PlatformContractInstance,
    ProjectContractInstance,
    ProjectInitMsg,
} from "./contracts";
import { banner, base64, entropy, minutesInBlocks, p, sleep } from "./utils";
import { exit } from "process";

const CHAIN_ID = "secretdev-1";

const printBalance = async (name: string, secretjs: SecretNetworkClient) => {
    const { balance: { amount }, } = await secretjs.query.bank.balance({
        address: secretjs.address,
        denom: "uscrt",
    });
      
    console.log(`${name} - ${secretjs.address} has ${Number(amount) / 1e6} SCRT!`);
}

const printBlock = async () : Promise<number> => {
    const latestBlockResponse = await a.signer.query.tendermint.getLatestBlock({});
    const block = parseInt(latestBlockResponse.block.header.height);
    p(block);
    return block;
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

    console.log("Seeding a, b, c, d with 100,000,000 sSCRT");
    await sscrt.deposit(a.signer, "100000000000000");
    await sscrt.deposit(b.signer, "100000000000000");
    await sscrt.deposit(c.signer, "100000000000000");
    await sscrt.deposit(d.signer, "100000000000000");

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

    console.log("Creating platform query permits for a, b, c, d");
    a.platform.queryPermit = await newPermit(a.wallet, a.wallet.address, CHAIN_ID, "test", [platform.address], ["owner"], false);
    p(a.platform.queryPermit);
    b.platform.queryPermit = await newPermit(b.wallet, b.wallet.address, CHAIN_ID, "test", [platform.address], ["owner"], false);
    p(b.platform.queryPermit);
    c.platform.queryPermit = await newPermit(c.wallet, c.wallet.address, CHAIN_ID, "test", [platform.address], ["owner"], false);
    p(c.platform.queryPermit);
    d.platform.queryPermit = await newPermit(d.wallet, d.wallet.address, CHAIN_ID, "test", [platform.address], ["owner"], false);
    p(d.platform.queryPermit);
}

const testProjectNoSnip24Reward = async (projectCode: ContractInfo) => {
    const platform = contracts.platform as PlatformContractInstance;
    contracts.project = new ProjectContractInstance("project", projectCode);
    const project = contracts.project as ProjectContractInstance; 
    const sscrt = contracts.sscrt as Snip20ContractInstance;
    let latestBlockResponse: GetLatestBlockResponse;
    let block: number;

    banner("Create project no snip24 reward");
    latestBlockResponse = await a.signer.query.tendermint.getLatestBlock({});
    block = parseInt(latestBlockResponse.block.header.height);

    let projectInitMsg: ProjectInitMsg = {
        creator: a.signer.address,
        title: "Project 1",
        subtitle: "Subtitle of project 1",
        description: "Description of project 1",
        cover_img: "http://example.com/img.png",
        pledged_message: "This is pledged message",
        funded_message: "This is funded message",
        reward_messages: [
            {
                threshold: "10000000", // 10 sscrt
                message: "10 sscrt club message",
            },
            {
                threshold: "100000000", // 100 sscrt
                message: "100 sscrt club message",
            },
        ],
        goal: "1000000000", // 1000 sscrt
        deadline: block + minutesInBlocks(2),
        deadman: 100000,
        categories: [1, 2, 3],
        source_contract: platform.address,
        source_hash: platform.contract.codeHash,
        snip20_contract: sscrt.address,
        snip20_hash: sscrt.contract.codeHash,
        entropy: entropy(),
        minimum_pledge: "1000000",
        maximum_pledge: "100000000",
        padding: "============================",
    };
    
    let tx = await project.instantiate(a.signer, projectInitMsg, "project" + entropy());
    console.log(tx);
}

const testSuccessfulProjectFromPlatformNoSnip24Reward = async (projectCode: ContractInfo) => {
    const platform = contracts.platform as PlatformContractInstance; 
    const sscrt = contracts.sscrt as Snip20ContractInstance;

    banner("Create project no snip24 reward");
    let block = await printBlock();
    
    let tx = await platform.create(a.signer, {
        title: "Project 1",
        subtitle: "Subtitle of project 1",
        description: "Description of project 1",
        cover_img: "http://example.com/img.png",
        pledged_message: "This is pledged message",
        funded_message: "This is funded message",
        reward_messages: [
            {
                threshold: "10000000", // 10 sscrt
                message: "10 sscrt club message",
            },
            {
                threshold: "100000000", // 100 sscrt
                message: "100 sscrt club message",
            }
        ],
        goal: "200000000", // 200 sscrt
        deadline: block + minutesInBlocks(2),
        categories: [1, 2, 3],
        snip20_contract: sscrt.address,
        snip20_hash: sscrt.contract.codeHash,
        entropy: entropy(),
        padding: "============================",
    });
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(tx.data[0]).data)));

    let projects = (await platform.queryProjects(b.signer)).projects.projects;
    p(projects);

    let project: ProjectContractInstance = new ProjectContractInstance("first project", projectCode, projects[0].address);

    banner("anon project status query");
    p(await project.queryStatus(a.signer));
    // all projects share platform query permit
    banner("owner project status query");
    p(await project.queryStatusPermit(a.signer, a.platform.queryPermit));
    banner("non-contributor project status query");
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    banner("b contributes 0.5 sscrt, not enough!");
    let send_tx = await sscrt.send(b.signer, project.address, "500000", 300_000);
    p(send_tx.rawLog);

    banner("b contributes 1 sscrt");
    send_tx = await sscrt.send(b.signer, project.address, "1000000", 300_000);
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    banner("b and c make comments");
    let comment_tx = await project.comment(b.signer, "Comment 1");
    comment_tx = await project.comment(c.signer, "Comment 2");
    p(await project.queryComments(b.signer));

    banner("b wants refund");
    p(await sscrt.queryBalance(b.signer, b.sScrt.queryPermit));
    let refund_tx = await project.refund(b.signer);
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(refund_tx.data[0]).data)));
    p(await sscrt.queryBalance(b.signer, b.sScrt.queryPermit));
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    banner("b contributes 10,000,000 sscrt, too much!")
    send_tx = await sscrt.send(b.signer, project.address, "10000000000000", 300_000);
    p(send_tx.rawLog);

    banner("c contributes 50 sscrt");
    send_tx = await sscrt.send(c.signer, project.address, "50000000", 300_000);
    p(await project.queryStatusPermit(c.signer, c.platform.queryPermit));

    banner("d contributes 175 sscrt");
    send_tx = await sscrt.send(d.signer, project.address, "175000000", 300_000);
    p(await project.queryStatusPermit(d.signer, d.platform.queryPermit));

    banner("b contributes 1 sscrt more");
    send_tx = await sscrt.send(b.signer, project.address, "1000000", 300_000);
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    banner("a wants pay out -- too early");
    await printBlock();
    let payout_tx = await project.payOut(a.signer);
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(payout_tx.data[0]).data)));
    
    await sleep(90_000);

    banner("b wants pay out -- invalid user");
    payout_tx = await project.payOut(b.signer);
    p(payout_tx.rawLog);

    banner("a wants pay out -- ok now");
    await printBlock();
    p(await sscrt.queryBalance(a.signer, a.sScrt.queryPermit));
    payout_tx = await project.payOut(a.signer);
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(payout_tx.data[0]).data)));
    p(await sscrt.queryBalance(a.signer, a.sScrt.queryPermit));

    banner("a tries second pay out -- fails");
    payout_tx = await project.payOut(a.signer);
    p(payout_tx.rawLog);

    banner("b queries status - sees funded message");
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    banner("c queries status - sees funded message and one reward message");
    p(await project.queryStatusPermit(c.signer, c.platform.queryPermit));

    banner("d queries status - sees funded message and two reward messages");
    p(await project.queryStatusPermit(d.signer, d.platform.queryPermit));

}

const testUnsuccessfulProjectFromPlatformNoSnip24Reward = async (projectCode: ContractInfo) => {
    const platform = contracts.platform as PlatformContractInstance; 
    const sscrt = contracts.sscrt as Snip20ContractInstance;

    banner("Create project no snip24 reward -- unsuccessful");
    let block = await printBlock();
    
    let tx = await platform.create(a.signer, {
        title: "Project 2",
        subtitle: "Subtitle of project 2",
        description: "Description of project 2",
        cover_img: "http://example.com/img.png",
        pledged_message: "This is pledged message",
        funded_message: "This is funded message",
        reward_messages: [
            {
                threshold: "10000000", // 10 sscrt
                message: "10 sscrt club message",
            },
            {
                threshold: "100000000", // 100 sscrt
                message: "100 sscrt club message",
            }
        ],
        goal: "200000000", // 200 sscrt
        deadline: block + 3,
        categories: [1, 2, 3],
        snip20_contract: sscrt.address,
        snip20_hash: sscrt.contract.codeHash,
        entropy: entropy(),
        padding: "============================",
    });
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(tx.data[0]).data)));

    let projects = (await platform.queryProjects(b.signer)).projects.projects;
    p(projects);

    let project: ProjectContractInstance = new ProjectContractInstance("first project", projectCode, projects[0].address);

    banner("anon project status query");
    p(await project.queryStatus(a.signer));
    // all projects share platform query permit
    banner("owner project status query");
    p(await project.queryStatusPermit(a.signer, a.platform.queryPermit));
    banner("non-contributor project status query");
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    banner("b contributes 1 sscrt");
    let send_tx = await sscrt.send(b.signer, project.address, "1000000", 400_000);
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(send_tx.data[0]).data)));
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    await sleep(20_000);

    banner("b contributes 1 sscrt after expired");
    send_tx = await sscrt.send(b.signer, project.address, "1000000", 400_000);
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(send_tx.data[0]).data)));
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

    banner("a wants pay out -- but not successful");
    await printBlock();
    p(await sscrt.queryBalance(a.signer, a.sScrt.queryPermit));
    let payout_tx = await project.payOut(a.signer);
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(payout_tx.data[0]).data)));
    p(await sscrt.queryBalance(a.signer, a.sScrt.queryPermit));

    banner("b wants refund");
    p(await sscrt.queryBalance(b.signer, b.sScrt.queryPermit));
    let refund_tx = await project.refund(b.signer);
    p(JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(refund_tx.data[0]).data)));
    p(await sscrt.queryBalance(b.signer, b.sScrt.queryPermit));
    p(await project.queryStatusPermit(b.signer, b.platform.queryPermit));

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
    p(contracts.sscrt);

    let projectContractInfo = await setupProjectContract(a.signer);
    p(projectContractInfo);
    await setupPlatformContract(a.signer, projectContractInfo);
    p(contracts.platform);
   
    //await testSuccessfulProjectFromPlatformNoSnip24Reward(projectContractInfo);
    //await testUnsuccessfulProjectFromPlatformNoSnip24Reward(projectContractInfo);

    console.log("DONE");
}

(async () => {
    main();
})();