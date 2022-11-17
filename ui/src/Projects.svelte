<script lang="ts">
    import { fade } from 'svelte/transition';
    import Pagination from './Pagination.svelte';
    import { push } from 'svelte-spa-router';
	import { toast } from '@zerodevx/svelte-toast';
    import { holdForKeplr } from './lib/wallet';
    import type { KeplrStore } from './stores/keplr';
    import { ContractInfo, } from './lib/contract';
    import ProjectPreview from './ProjectPreview.svelte';
    import { PlatformContractInstance, } from './lib/contracts';
    import { PLATFORM_CODE_HASH, PLATFORM_CONTRACT, } from './lib/env';

    const platform: PlatformContractInstance = new PlatformContractInstance("platform", PLATFORM_CODE_HASH, PLATFORM_CONTRACT);

    interface ProjectsParams {
        page?: string;
    };

    export let params: ProjectsParams = {};
    const pageSize = 5;

    function getPageValue(page) {
        if (page) {
            return parseInt(page);
        } else {
            return 0;
        }
    }

    $: pageValue = getPageValue(params.page);

    let projectCount: number = 0;

    function changePage(page: number) {
		push(`/projects/${page-1}`);
	}

    $: if (params.page) { // watch the params.page for changes
        loadProjects(); // reload projects data
    }

    let keplr: KeplrStore;

    let projects: ContractInfo[] = [];

    async function loadProjects() {
		keplr = await holdForKeplr(keplr);
        const { scrtClient } = keplr;
        const result = await platform.queryProjects(scrtClient, pageValue, pageSize);
        //console.log(result);
		if (result.projects) {
            projectCount = result.projects.count;
            projects = result.projects.projects;
		} else {
			toast.push("Error querying for projects");
		}
	}
</script>

<svelte:head>
	<title>Mage Hands: Projects</title>
</svelte:head>

<section in:fade="{{duration: 500}}">
    <div class="projects">
        <h1>Projects</h1>
        {#each projects as project (project)}
            <ProjectPreview project={project} />
        {/each}
        {#if projectCount > pageSize}
    	    <Pagination
      		    current_page={pageValue+1}
      		    from={pageValue*pageSize+1}
			    last_page={Math.floor(projectCount/pageSize)+1}
      		    per_page={pageSize}
      		    to={pageValue*pageSize+pageSize}
      		    total={projectCount}
      		    on:change="{(ev) => changePage(ev.detail)}" 
            />
  	    {/if}
    </div>
</section>

