<script lang="ts">
	import { scale } from 'svelte/transition';
    import { toast } from '@zerodevx/svelte-toast';
	import { push } from 'svelte-spa-router';
	import { getSignature, KeplrStore } from './stores/keplr';
	import { CHAIN_ID } from './lib/env';
	import { permitName } from './stores/permits';
	import { holdForKeplr } from './lib/wallet';
    import { ContractInfo, } from './lib/contract';
	import Paper from '@smui/paper';

	import { permitsStore, } from './stores/permits';
    import { ProjectStatusResult, ProjectContractInstance, } from './lib/contracts';
	import { PLATFORM_CONTRACT } from './lib/env';
    import { SecretNetworkClient, Permit } from 'secretjs';
	import { getBlock } from './lib/utils';
    import LayoutGrid from '@smui/layout-grid';
    import ProjectPreviewCells from './ProjectPreviewCells.svelte';

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
			//console.log(permits);
			if (permits[scrtClient.address]) {
				permit = permits[scrtClient.address];
			} else {
				try {
					let amino = await getSignature(CHAIN_ID);
                    let signedPermit = amino.signed.msgs[0].value;
					permit = {
						params: {
							allowed_tokens: signedPermit.allowed_tokens,
							chain_id: CHAIN_ID,
							permit_name: signedPermit.permit_name,
							permissions: signedPermit.permissions,
						},
						signature: amino.signature,
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
			console.log(projectStatus);
			goalNum = parseFloat(projectStatus.goal) / 1000000;
            totalNum = parseFloat(projectStatus.total) / 1000000;
        }
    }

    loadProject();

	function handleProjectClick() {
		push(`/project/${project.address}/${project.code_hash}`);
	}
</script>

{#if projectStatus}
	<div class="lgmargin" transition:scale|local={{ start: 0.7 }}>
		<Paper transition elevation={6} on:click={handleProjectClick}>
			<LayoutGrid>
				<ProjectPreviewCells bind:projectStatus bind:currentBlock bind:totalNum bind:goalNum />
			</LayoutGrid>
		</Paper>
	</div>
{/if}

<style lang="scss">
    .lgmargin {
        margin: 0 0 2.5rem 0;
    }
</style>
