<script lang="ts">
    export let current_page: number;
    export let last_page: number;
    export let per_page: number;
    export let from: number;
    export let to: number;
    export let total :number;
  
    import { createEventDispatcher } from 'svelte';
  
    const dispatch = createEventDispatcher();
  
    function range(size, startAt = 0) {
        return [...Array(size).keys()].map(i => i + startAt);
    }
  
    function changePage(page) {
        if (page !== current_page) {
            dispatch('change', page);
        }
    }
</script>
  
<p>
    Page <code>{current_page}</code> of <code>{last_page}</code> (<code>{from}</code> - <code>{Math.min(to,total)}</code> on <code>{total}</code> projects)
</p>
  
<nav class="pagination">
    <ul>
        <li class="{current_page === 1 ? 'disabled' : ''}">
            <a href="javascript:void(0)" on:click="{() => changePage(current_page - 1)}">
                <span aria-hidden="true">«</span>
            </a>
        </li>
        {#each range(last_page, 1) as page}
            <li class="{page === current_page ? 'active': ''}">
                <a href="javascript:void(0)" on:click="{() => changePage(page)}">{page}</a>
            </li>
        {/each}
        <li class="{current_page === last_page ? 'disabled' : ''}">
            <a href="javascript:void(0)" on:click="{() => changePage(current_page + 1)}">
                <span aria-hidden="true">»</span>
            </a>
        </li>
    </ul>
</nav>

<style>
    .pagination {
        display: flex;
        justify-content: center;
    }
    .pagination ul {
        display: flex;
        padding-left: 0;
        list-style: none;
    }
    .pagination li a {
        position: relative;
        display: block;
        padding: .5rem .75rem;
        margin-left: -1px;
        line-height: 1.25;
        background-color: #fff;
        color: var(--primary-color);
        border: 1px solid #dee2e6;
    }
    .pagination li.active a {
        color: #fff;
        background-color: var(--primary-color);
        border-color: var(--primary-color);
    }
    .pagination li.disabled a {
        color: #6c757d;
        pointer-events: none;
        cursor: auto;
        border-color: #dee2e6;
    }
</style>