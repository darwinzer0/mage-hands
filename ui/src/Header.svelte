<script lang="ts">
	import { onMount } from 'svelte';
    import { location } from 'svelte-spa-router';
	import { keplrStore } from './stores/keplr';
	import type { KeplrStore } from './stores/keplr';
	import SvelteTooltip from 'svelte-tooltip';

	let keplr: KeplrStore;
	onMount( async () => {
		keplrStore.subscribe(value => {
			keplr = value;
		});
	});

	$: scrtAuthorized = keplr && keplr.scrtAuthorized;
</script>

<header>
	<div class="corner">
		<img src="/dendrites-logo.png" alt="Mage Hands" />
	</div>

	<nav>
		<svg viewBox="0 0 2 3" aria-hidden="true">
			<path d="M0,0 L1,2 C1.5,3 1.5,3 2,3 L2,0 Z" />
		</svg>
		<ul>
			<li class:active={!($location.startsWith('/project') || $location.startsWith('/newproj'))}><a href="#/">Home</a></li>
			<li class:active={$location.startsWith('/projects')}><a href="#/projects/0">Projects</a></li>
			<li class:active={$location.startsWith('/newproj')}><a href="#/newproj">Create</a></li>
		</ul>
		<svg viewBox="0 0 2 3" aria-hidden="true">
			<path d="M0,0 L0,3 C0.5,3 0.5,3 1,2 L2,0 Z" />
		</svg>
	</nav>

	<div class="corner">
		{#if scrtAuthorized}
			<SvelteTooltip tip={keplr.scrtClient.address} left >
				<img src="wallet.png" alt="Wallet available"/>
			</SvelteTooltip>
		{:else}
			<img src="wallet_bw.png" alt="Keplr not available"/>
		{/if}
	</div>
</header>

<style lang="scss">
	header {
		display: flex;
		justify-content: space-between;
	}

	.corner {
		width: 3em;
		height: 3em;
	}

	.corner img {
		width: 2em;
		height: 2em;
		object-fit: contain;
	}

	nav {
		display: flex;
		justify-content: center;
		--background: rgba(255, 255, 255, 0.9);
	}

	svg {
		width: 2em;
		height: 3em;
		display: block;
	}

	path {
		fill: var(--background);
	}

	ul {
		position: relative;
		padding: 0;
		margin: 0;
		height: 3em;
		display: flex;
		justify-content: center;
		align-items: center;
		list-style: none;
		background: var(--background);
		background-size: contain;
	}

	li {
		position: relative;
		height: 100%;
	}

	li.active::before {
		--size: 6px;
		content: '';
		width: 0;
		height: 0;
		position: absolute;
		top: 0;
		left: calc(50% - var(--size));
		border: var(--size) solid transparent;
		border-top: var(--size) solid var(--accent-color-dark);
	}

	nav a {
		display: flex;
		height: 100%;
		align-items: center;
		padding: 0 1em;
		color: var(--heading-color);
		font-weight: 700;
		font-size: 0.8rem;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		text-decoration: none;
		transition: color 0.2s linear;
	}

	a:hover {
		color: var(--accent-color-dark);
	}
</style>