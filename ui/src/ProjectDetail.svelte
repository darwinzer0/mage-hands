<script lang="ts">
    import { CHAIN_ID, getSignature, KeplrStore } from './stores/keplr';
    import { ProjectStatusResult, ProjectContractInstance, PLATFORM_CONTRACT, } from './lib/contracts';
    import Button, { Label } from '@smui/button';

    let keplr: KeplrStore;

    let projectStatus: ProjectStatusResult = null;
    let currentBlock: number = null;

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

    .submit {
		margin: auto;
	}
</style>