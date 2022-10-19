<script lang="ts">
    import { fade, scale } from 'svelte/transition';
	import { flip } from 'svelte/animate';
    import Pagination from './Pagination.svelte';
    import { push } from 'svelte-spa-router';
	import { toast } from '@zerodevx/svelte-toast';
    import { holdForKeplr } from './lib/wallet';
    import type { KeplrStore } from './stores/keplr';
    import { ContractInfo, } from './lib/contract';
    import ProjectPreview from './ProjectPreview.svelte';
    import { PlatformContractInstance, PLATFORM_CODE_HASH, PLATFORM_CONTRACT} from './lib/contracts';

    const platform: PlatformContractInstance = new PlatformContractInstance("platform", PLATFORM_CODE_HASH, PLATFORM_CONTRACT);

    interface ProjectsParams {
        page?: string;
    };

    export let params: ProjectsParams = {};
    const pageSize = 10;

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

    let keplr: KeplrStore;

    let projects: ContractInfo[] = [];

    async function loadProjects() {
		keplr = await holdForKeplr(keplr);
        const { scrtClient } = keplr;
        const result = await platform.queryProjects(scrtClient, pageValue, pageSize);
        console.log(result);
		if (result.projects) {
            projectCount = result.projects.count;
            projects = result.projects.projects;
		} else {
			toast.push("Error querying for projects");
		}
	}
    loadProjects();
</script>


<svelte:head>
	<title>Mage Hands: Projects</title>
</svelte:head>

<section in:fade="{{duration: 500}}">
    <div class="projects">
        <h1>Fundraising Projects</h1>
        {#each projects as project (project)}
            <div 
                transition:scale|local={{ start: 0.7 }}
                animate:flip={{ duration: 200 }}
                class="lgmargin"
            >
                <ProjectPreview project={project} />
            </div>
        {/each}
        {#if projectCount > pageSize}
    	    <Pagination
      		    current_page={pageValue+1}
      		    from={pageValue*pageSize+1}
			    last_page={Math.floor(projectCount/pageSize)+1}
      		    per_page={pageSize}
      		    to={pageValue*pageSize+10}
      		    total={projectCount}
      		    on:change="{(ev) => changePage(ev.detail)}" 
            />
  	    {/if}
    </div>
    <p>Projects page {+params.page + 1}</p>
</section>

<style lang="scss">
    .lgmargin {
        margin: 0 0 2.5rem 0;
    }
</style>
