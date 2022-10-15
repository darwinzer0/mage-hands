<script lang="ts">
    import { toast } from '@zerodevx/svelte-toast';
	import { CHAIN_ID, getSignature, KeplrStore } from './stores/keplr';
	import { permitName } from './stores/permits';
	import { holdForKeplr } from './lib/wallet';
    import { ContractInfo, } from './lib/contract';
    import { allCategories } from './lib/categories';

    import Chip, { Set, Text } from '@smui/chips';
	import Paper from '@smui/paper';

	import { permitsStore, } from './stores/permits';
    import { ProjectStatusResult, ProjectContractInstance, PLATFORM_CONTRACT, } from './lib/contracts';
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
			if (permit) {
				const queryMsg = { status_with_permit: { permit } };
				projectStatus = await projectContract.queryStatusPermit(scrtClient, permit);
				toast.push(JSON.stringify(projectStatus));
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

	function handleProjectClick() {
		toast.push("Clicked the paper");
	}

    function categoryLabels(categories: number[]) {
		return categories.map( category => {
			return allCategories[category];
		})
	}
</script>

{#if projectStatus}
	<Paper transition elevation={4} on:click={handleProjectClick}>
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
		<pre>Creator: {projectStatus.creator}</pre>
		<h1>{projectStatus.title}</h1>
		<h2>{projectStatus.subtitle}</h2>
		<h3>Deadline: {projectStatus.deadline}</h3>
		<h3>{totalNum} out of {goalNum} SCRT funded</h3>
		{#if projectStatus.contribution}
			<h3>Your contribution: {projectStatus.contribution} sSCRT</h3>
		{/if}
		<Set chips={categoryLabels(projectStatus.categories)} let:chip nonInteractive>
			<Chip {chip}>
				<Text>{chip}</Text>
			</Chip>
		</Set>
	</Paper>
{/if}

<div>{project.address}</div>
<div>{project.code_hash}</div>

<style>

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

	* :global(.smui-paper) {
		background-color: #ffffff08;
	}
</style>