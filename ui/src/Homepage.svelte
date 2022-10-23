<script lang="ts">
    import { push } from 'svelte-spa-router';
    import { fade } from 'svelte/transition';
	import { CHAIN_ID, getSignature } from './stores/keplr';

	let signature;
	
/*
	function handleGetSignature() {
		getSignature(CHAIN_ID).then( sig => { 
			console.log(sig);
			signature = sig;
		});
	}
*/	
</script>

<svelte:head>
	<title>Mage Hands</title>
</svelte:head>

<section in:fade="{{duration: 500}}">

	<h1>
        Mage Hands
    </h1>

    <div class="welcome">
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <picture on:click={() => {push('/projects/0');}}>
            <!-- <source srcset="magehands.webp" type="image/webp" /> -->
            <img src="dendrites.png" alt="Mage Hands" />
        </picture>
    </div>

    <h2>
        Crowdfunding on Secret Network
    </h2>
<!--
	<button on:click={handleGetSignature}>
		Get Signature
	</button>
	{#if signature}
	    signature
	{/if}
-->
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 1;
	}

	h1 {
		width: 100%;
		color: var(--heading-secondary-color);
		font-family: var(--font-wizzta);
		font-size: 72px;
	}

	h2 {
		color: var(--heading-secondary-color);
		font-size: 24px;
	}

	.welcome {
		position: relative;
		width: 100%;
		height: 0;
		/* padding: 0 0 calc(100% * 517 / 2048) 0; */
		padding: 0 0 calc(100% * 600 / 2048) 0;
        margin-bottom: 1rem;
	}

	.welcome img {
		position: absolute;
		height: 100%;
		width: 100%;
		top: 0;
		display: block;
	}
</style>
