<script lang="ts">
    import { toast } from '@zerodevx/svelte-toast';
	import { CHAIN_ID, getSignature, KeplrStore } from './stores/keplr';
	import { permitName } from './stores/permits';
	import { holdForKeplr } from './lib/wallet';
    import { PLATFORM_CONTRACT, ContractInfo, } from './lib/contract';
    import { allCategories } from './lib/categories';

    import Chip, { Set, Text } from '@smui/chips';
	import Button, { Label } from '@smui/button';

	import { permitsStore, } from './stores/permits';
    import { ProjectStatusResult, ProjectContractInstance } from './lib/contracts';
    import { SecretNetworkClient, Permit } from 'secretjs';
	import { getBlock } from './lib/utils';

    let keplr: KeplrStore;

    export let project: ContractInfo;

	const projectContract: ProjectContractInstance = new ProjectContractInstance("project-"+project.address, project.code_hash, project.address);
    let projectStatus: ProjectStatusResult = null;
	let goalNum: number = null;
	let totalNum: number = null;
	let scrtClient: SecretNetworkClient = null;
	let currentBlock: number = null;

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
			console.log(permits);
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
			console.log(permit);
			if (permit) {
				const queryMsg = { status_with_permit: { permit } };
				projectStatus = await projectContract.queryStatusPermit(scrtClient, permit);
				goalNum = parseFloat(projectStatus.goal) / 1000000;
            	totalNum = parseFloat(projectStatus.total) / 1000000;
			} else {
				const queryMsg = { status: {} };
				projectStatus = await projectContract.queryStatus(scrtClient);
				goalNum = parseFloat(projectStatus.goal) / 1000000;
            	totalNum = parseFloat(projectStatus.total) / 1000000;
			}
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

    function categoryLabels(categories: number[]) {
		return categories.map( category => {
			return allCategories[category];
		})
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
<div
	class="project"
	class:projectSuccessful={projectStatus.status === "successful"}
	class:projectExpired={projectStatus.status === "expired" || (projectStatus.status === "fundraising" && currentBlock >= projectStatus.deadline)}
	class:projectFundraising={projectStatus.status === "fundraising" && currentBlock < projectStatus.deadline}
>
	{#if projectStatus.status === "successful"}
		<h1 class="successful">🎉 Successful 🎉</h1>
	{/if}
	{#if projectStatus.status === "fundraising"}
		{#if currentBlock > projectStatus.deadline}
			<h1 class="expired">Not funded</h1>
		{:else}
			<h1 class="fundraising">Fundraising</h1>
		{/if}
	{/if}
	{#if projectStatus.status === "expired"}
		<h1 class="expired">Not funded</h1>
	{/if}
	<pre>Creator: {projectStatus.creator}</pre>
	<h2>{projectStatus.title}</h2>
	<h3>Deadline: {projectStatus.deadline.toLocaleString()}</h3>
	<h3>{totalNum} out of {goalNum} SCRT funded</h3>
	{#if projectStatus.contribution}
		<h3>Your contribution: {projectStatus.contribution} SCRT</h3>
	{/if}
	<Set chips={categoryLabels(projectStatus.categories)} let:chip nonInteractive>
		<Chip {chip}>
			<Text>{chip}</Text>
		</Chip>
	</Set>
	<h4>Description</h4>
	<p>{projectStatus.description}</p>
	{#if projectStatus.pledged_message}
		<h4>Pledged message</h4>
		<p>{projectStatus.pledged_message}</p>
	{/if}
	{#if projectStatus.funded_message}
		<h4>Funded message</h4>
		<p>{projectStatus.funded_message}</p>
	{/if}
	{#if projectStatus.reward_messages.length > 0}
		<h4>Reward messages</h4>
		{#each projectStatus.reward_messages as rewardMessage (rewardMessage)}
			<p>{rewardMessage}</p>
		{/each}
	{/if}
	<div class="submit">
		{#if !doneProject(projectStatus)}
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
</div>
{/if}

<div>{project.address}</div>
<div>{project.code_hash}</div>

<style>

	.project {
		margin: 0 0 2.0rem 0;
		padding: 0.5rem;
		background-color: var(--primary-color);
		border-radius: 8px;
		filter: drop-shadow(2px 4px 6px rgba(0, 0, 0, 0.1));
		transform: translate(-1px, -1px);
		transition: filter 0.2s, transform 0.2s;
	}

	.projectSuccessful {
		border-color: lightgreen;
	}

	.projectFundraising {
		border-color: var(--accent-color);
	}

	.projectExpired {
		border-color: lightgrey;
	}

	.project h2 {
		margin-left: 0.5rem;
		font-size: 28px;
	}

	.project h3 {
		margin-left: 0.5rem;
		font-size: 16px;
	}

	.project h4 {
		margin-left: 0.5rem;
		font-size: 14px;
	}

	.project p {
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

	* :global(.shaped-outlined),
	* :global(.shaped-outlined .mdc-select__anchor) {
    	border-radius: 28px;
  	}
  	* :global(.shaped-outlined .mdc-text-field__input) {
    	padding-left: 32px;
    	padding-right: 0;
  	}
  	* :global(.shaped-outlined
      	.mdc-notched-outline
      	.mdc-notched-outline__leading) {
		border-radius: 28px 0 0 28px;
		width: 28px;
  	}
  	* :global(.shaped-outlined
      	.mdc-notched-outline
    	.mdc-notched-outline__trailing) {
		border-radius: 0 28px 28px 0;
  	}
	* :global(.shaped-outlined .mdc-notched-outline .mdc-notched-outline__notch) {
    	max-width: calc(100% - 28px * 2);
  	}
  	* :global(.shaped-outlined.mdc-select--with-leading-icon
    	.mdc-notched-outline:not(.mdc-notched-outline--notched)
    	.mdc-floating-label) {
    	left: 16px;
  	}
</style>