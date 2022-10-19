<script lang="ts">
	import { onMount } from 'svelte';
	import Router from "svelte-spa-router";
	import { keplrStore } from './stores/keplr';

	import Projects from './Projects.svelte';
	import NewProject from './NewProject.svelte';
	import Homepage from './Homepage.svelte';
	import Header from "./Header.svelte";

	import { SvelteToast } from '@zerodevx/svelte-toast';
    import ProjectDetail from './ProjectDetail.svelte';

	const routes = {
    	'/newproj/': NewProject,
    	'/projects/:page?': Projects,
		'/project/:contract/:hash': ProjectDetail,
    	'*': Homepage,
	}

	onMount( async () => {
		await keplrStore.connect();
	});
</script>

<div id="svelte">
	<Header />
	<main>
		<Router { routes } />
	</main>
	<footer>
		<p>[Terms of use]</p>
	</footer>
</div>

<SvelteToast />

<style lang="scss">
	#svelte {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 1rem;
		width: 100%;
		max-width: 1366px;
		margin: 0 auto;
		box-sizing: border-box;
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 40px;
	}

	@media (min-width: 480px) {
		footer {
			padding: 40px 0;
		}
	}
</style>
