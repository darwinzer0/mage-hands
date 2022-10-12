<script lang="ts">
    import { fade } from 'svelte/transition';
	import { CHAIN_ID, getSignature } from './stores/keplr';

	let signature;
	
	function handleClick() {
		getSignature(CHAIN_ID).then( sig => { 
			console.log(sig);
			signature = sig;
		});
	}
	
</script>

<svelte:head>
	<title>Mage Hands</title>
</svelte:head>

<section in:fade="{{duration: 500}}">
    <div class="welcome">
        <picture>
            <source srcset="magehands.webp" type="image/webp" />
            <img src="magehands.png" alt="Mage Hands" />
        </picture>
    </div>
    <h1>
        Mage Hands
    </h1>

    <h2>
        Crowdfunding on Secret Network
    </h2>

	<button on:click={handleClick}>
		Get Signature
	</button>
	{#if signature}
	    signature
	{/if}
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
		font-size: 60px;
	}

	h2 {
		color: var(--heading-secondary-color);
		font-size: 24px;
	}

	.welcome {
		position: relative;
		width: 100%;
		height: 0;
		padding: 0 0 calc(100% * 517 / 2048) 0;
        margin-top: 1rem;
	}

	.welcome img {
		position: absolute;
		height: 100%;
		width: 100%;
		top: 0;
		display: block;
	}
</style>
