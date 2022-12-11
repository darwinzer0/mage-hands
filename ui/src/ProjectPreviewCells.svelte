<script lang="ts">
	import { categoryLabels } from './lib/categories';
    import Chip, { Set, Text } from '@smui/chips';
	import { timeUntilDeadline } from './lib/utils';
    import { Cell } from '@smui/layout-grid';

    export let projectStatus;
    export let currentBlock;
    export let totalNum;
    export let goalNum;
</script>

<Cell span={12} align="middle">
    {#if projectStatus.status === "successful"}
        <h1 class="successful banner">🎉 Successful 🎉</h1>
    {:else if projectStatus.status === "fundraising"}
        {#if currentBlock > projectStatus.deadline}
            <h1 class="expired banner">Unsuccessful</h1>
        {:else}
            <h1 class="fundraising banner">Fundraising</h1>
        {/if}
    {:else if projectStatus.status === "expired"}
        <h1 class="expired">Not funded</h1>
    {/if}
</Cell>
<Cell span={4} align="bottom">
    <div class="coverimg">
        <!-- svelte-ignore a11y-missing-attribute -->
        <img src={projectStatus.cover_img} />
    </div>
</Cell>
<Cell span={8} align="bottom">
    <h1 class="title">{projectStatus.title}</h1>
    <h2>{projectStatus.subtitle}</h2>
    <div class="catchips">
        <Set chips={categoryLabels(projectStatus.categories)} let:chip nonInteractive>
            <Chip {chip}>
                <Text>{chip}</Text>
            </Chip>
        </Set>
    </div>
</Cell>
<Cell span={12}>
    <div>
        <p>Creator: {projectStatus.creator}</p>
        <h3>Deadline: {timeUntilDeadline(currentBlock, projectStatus.deadline)}</h3>
        <h3>{totalNum} out of {goalNum} sSCRT funded</h3>
        {#if projectStatus.contribution}
            <h3>Your contribution: {parseFloat(projectStatus.contribution) / 1000000} sSCRT</h3>
        {/if}
    </div>
</Cell>

<style>
	.title {
		text-align: left;
	}

	h3 {
		font-family: Raleway, sans-serif;
		font-size: 1rem;
		font-weight: 700;
	}
/*
	.coverimg {
		display: flex;
    	justify-content: flex-end;
		width: 200px;
	}
*/
	.banner {
		text-align: center;
		font-size: 20px;
		font-weight: 700;
		color: white;
		padding: 0.5rem;
		box-shadow: 3px 3px inset rgba(0, 0, 0, 0.5);
	}

	.fundraising {
		background-color: var(--tertiary-color);
	}

	.successful {
		background-color: var(--primary-color);
	}

	.expired {
		background-color: var(--secondary-color);
	}

	.catchips {
		margin-left: -8px;
	}
</style>