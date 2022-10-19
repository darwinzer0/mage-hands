<script lang="ts">
    import { toast } from '@zerodevx/svelte-toast';
    import { CHAIN_ID, getSignature, KeplrStore } from './stores/keplr';
    import { permitName } from './stores/permits';
	import { holdForKeplr } from './lib/wallet';
    import { ContractInfo, } from './lib/contract';
    import { categoryLabels } from './lib/categories';
    import Chip, { Set, Text } from '@smui/chips';
	import Paper from '@smui/paper';
    import { permitsStore, } from './stores/permits';
    import { ProjectStatusResult, ProjectContractInstance, PLATFORM_CONTRACT, } from './lib/contracts';
    import Button, { Label } from '@smui/button';
    import { SecretNetworkClient, Permit } from 'secretjs';
	import { getBlock, timeUntilDeadline } from './lib/utils';
    import Editor from './Editor.svelte';
    import pako from "pako";

    interface ProjectParams {
        contract: string;
        hash: string;
    };

    export let params: ProjectParams = null;

    let keplr: KeplrStore;
	const projectContract: ProjectContractInstance = new ProjectContractInstance("project-"+params.contract, params.hash, params.contract);

    let projectStatus: ProjectStatusResult = null;
    let goalNum: number = null;
	let totalNum: number = null;
    let scrtClient: SecretNetworkClient = null;
    let currentBlock: number = null;
    let descriptionFromPako = null;
    let pledgedMessageFromPako = null;
    let fundedMessageFromPako = null;
    let rewardMessagesFromPako = [];

    let permits;
	permitsStore.subscribe(value => {
		permits = value;
	});

    async function loadProject() {
		keplr = await holdForKeplr(keplr);
        scrtClient = keplr.scrtClient;
		currentBlock = await getBlock(scrtClient);
        if (scrtClient) {
			let permit: Permit;
			if (permits[scrtClient.address]) {
				permit = permits[scrtClient.address];
			} else {
				try {
					let signature = await getSignature(CHAIN_ID);
					permit = {
						params: {
							allowed_tokens: [PLATFORM_CONTRACT],
							chain_id: CHAIN_ID,
							permit_name: permitName,
							permissions: ["owner"],
						},
						signature
					}
					permitsStore.set({...permits, [scrtClient.address]: permit});
				} catch (err) {
					toast.push(err.toString());
				}
			}
			if (permit) {
				const queryMsg = { status_with_permit: { permit } };
				projectStatus = await projectContract.queryStatusPermit(scrtClient, permit);
			} else {
				const queryMsg = { status: {} };
				projectStatus = await projectContract.queryStatus(scrtClient);
			}
            goalNum = parseFloat(projectStatus.goal) / 1000000;
            totalNum = parseFloat(projectStatus.total) / 1000000;
            descriptionFromPako = JSON.parse(pako.ungzip(atob(projectStatus.description), { to: 'string' }));
            if (projectStatus.pledged_message) {
                pledgedMessageFromPako = JSON.parse(pako.ungzip(atob(projectStatus.pledged_message), { to: 'string' }));
            }
            if (projectStatus.funded_message) {
                fundedMessageFromPako = JSON.parse(pako.ungzip(atob(projectStatus.funded_message), { to: 'string' }));
            }
            rewardMessagesFromPako = projectStatus.reward_messages.map( rm => {
                return {
                    threshold: parseFloat(rm.threshold) / 1000000,
                    message: JSON.parse(pako.ungzip(atob(rm.message), { to: 'string' }))
                };
            });
        }
    }

    loadProject();

    function doneProject(proj: ProjectStatusResult) {
		return currentBlock > proj.deadline || proj.status === "expired" || proj.paid_out;
	}

    function isProjectCreator(creator: string) {
		const { scrtClient } = keplr;
		return scrtClient && scrtClient.address === creator;
	}

    function readyForPayout(proj: ProjectStatusResult) {
		return currentBlock > proj.deadline && proj.status === "successful" && !proj.paid_out;
	}

    async function handleContribute() {
        console.log("contribute");
	}

    async function handleRefund() {
        console.log("refund");
    }

    async function handlePayOut() {
        console.log("payout");
    }
</script>

{#if projectStatus}
    <Paper transition elevation={4}>
        {#if projectStatus.status === "successful"}
            <h1 class="successful">🎉 Successful 🎉</h1>
        {:else if projectStatus.status === "fundraising"}
            {#if currentBlock > projectStatus.deadline}
                <h1 class="expired">Unsuccessful</h1>
            {:else}
                <h1 class="fundraising">Fundraising</h1>
        {/if}
        {:else if projectStatus.status === "expired"}
            <h1 class="expired">Not funded</h1>
        {/if}
        <h1>{projectStatus.title}</h1>
        <h2>{projectStatus.subtitle}</h2>
        <p>Creator: {projectStatus.creator}</p>
        <h3>Deadline: {timeUntilDeadline(currentBlock, projectStatus.deadline)}</h3>
        <h3>{totalNum} out of {goalNum} SCRT funded</h3>
        {#if projectStatus.contribution}
            <h3>Your contribution: {projectStatus.contribution} sSCRT</h3>
        {/if}
        <Set chips={categoryLabels(projectStatus.categories)} let:chip nonInteractive>
            <Chip {chip}>
                <Text>{chip}</Text>
            </Chip>
        </Set>
        <h4>Description</h4>
        <div class="edmargin">
            <Editor data={descriptionFromPako} editorId="descriptionReader" readOnly={true} />
        </div>
        {#if pledgedMessageFromPako}
            <h4>Pledged message</h4>
            <div class="edmargin">
                <Editor data={pledgedMessageFromPako} editorId="pledgeMessageReader" readOnly={true} />
            </div>
        {/if}
        {#if fundedMessageFromPako}
            <h4>Funded message</h4>
            <div class="edmargin">
                <Editor data={fundedMessageFromPako} editorId="fundedMessageReader" readOnly={true} />
            </div>
        {/if}
        {#if rewardMessagesFromPako.length > 0}
            <h4>Reward messages</h4>
            {#each rewardMessagesFromPako as rewardMessage, i}
                <div class="edmargin">
                    <Editor data={rewardMessage} editorId={"rewardMessageReader"+i} readOnly={true} />
                </div>
            {/each}
        {/if}
        <div class="submit">
            {#if !doneProject(projectStatus) && !isProjectCreator(projectStatus.creator)}
                <Button 
                    on:click={() => handleContribute()} 
                    variant="raised" 
                    class="button-shaped-notch"
                >
                    <Label>Contribute</Label>
                </Button>
            {/if}
            {#if projectStatus.contribution && projectStatus.status !== "successful"}
                <Button 
                    on:click={() => handleRefund()} 
                    variant="raised" 
                    class="button-shaped-notch"
                >
                    <Label>Refund</Label>
                </Button>
            {/if}
        </div>

        {#if isProjectCreator(projectStatus.creator) && readyForPayout(projectStatus)}
            <div class="submit">
                <Button 
                    on:click={() => handlePayOut()} 
                    variant="raised" 
                    class="button-shaped-notch"
                >
                    <Label>Pay out</Label>
                </Button>
            </div>
        {/if}
    </Paper>
{/if}

<style>

    h1 {
	    font-family: Raleway, sans-serif;
	    font-size: 2rem;
	    text-align: center;
    }

    h2 {
	    font-family: Raleway, sans-serif;
	    font-size: 1rem;
        text-align: center;
    }

    h3 {
		margin-left: 0.5rem;
		font-size: 16px;
	}

	h4 {
		margin-left: 0.5rem;
		font-size: 14px;
	}

	p {
		white-space: break-spaces;
		margin-left: 0.5rem;
	}

    .fundraising {
		font-size: 20px;
		background-color: var(--accent-color);
		color: black;
		padding: 0.5rem;
	}

	.successful {
		font-size: 20px;
		background-color: lightgreen;
		color: black;
		padding: 0.5rem;
	}

	.expired {
		font-size: 20px;
		background-color: lightgrey;
		color: black;
		padding: 0.5rem;
	}

    .submit {
		margin: auto;
	}

    .edmargin {
		margin: 1rem 0 0 0;
	}
</style>