<script lang="ts">
    import { scale } from 'svelte/transition';
    import { keplrStore } from "./stores/keplr";
	import { get } from 'svelte/store';
    import { toast } from '@zerodevx/svelte-toast';
    import { getSignature, KeplrStore } from './stores/keplr';
    import { CHAIN_ID } from './lib/env';
    import { permitName } from './stores/permits';
	import { holdForKeplr } from './lib/wallet';
	import Paper from '@smui/paper';
    import { permitsStore, } from './stores/permits';
    import { ProjectStatusResult, ProjectContractInstance, Snip20ContractInstance, ProjectComment, ProjectSnip24Info} from './lib/contracts';
    import { PLATFORM_CONTRACT, SSCRT_CODE_HASH, SSCRT_CONTRACT, } from './lib/env';
    import { Label } from '@smui/button';
    import { Input } from '@smui/textfield';
    import { SecretNetworkClient, Permit } from 'secretjs';
	import { getBlock } from './lib/utils';
    import Editor from './Editor.svelte';
    import pako from "pako";
    import LayoutGrid, { Cell } from '@smui/layout-grid';
    import ProjectPreviewCells from './ProjectPreviewCells.svelte';
    import InnerGrid from '@smui/layout-grid/src/InnerGrid.svelte';

    interface ProjectParams {
        contract: string;
        hash: string;
    };

    export let params: ProjectParams = null;

    let keplr: KeplrStore;
	const projectContract: ProjectContractInstance = new ProjectContractInstance("project-"+params.contract, params.hash, params.contract);
    const sscrt: Snip20ContractInstance = new Snip20ContractInstance("sscrt", SSCRT_CODE_HASH, SSCRT_CONTRACT);

    let projectStatus: ProjectStatusResult = null;
    let goalNum: number = null;
	let totalNum: number = null;
    let scrtClient: SecretNetworkClient = null;
    let currentBlock: number = null;
    let descriptionFromPako = null;
    let snip24Info: ProjectSnip24Info = null;
    let totalInitialSupply = null;
    let contributorNumberOfTokens = null;
    let creatorNumberOfTokens = null;
    let pledgedMessageFromPako = null;
    let fundedMessageFromPako = null;
    let rewardMessagesFromPako = [];
    let rewardMessageFromPako = null;

    let contributionValue = "";

    let commentValue = "";
    let commentsLoaded = 0;
    let comments: ProjectComment[] = [];
    let loadedAllComments = false;

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
            goalNum = parseFloat(projectStatus.goal) / 1000000;
            totalNum = parseFloat(projectStatus.total) / 1000000;
            descriptionFromPako = JSON.parse(pako.ungzip(atob(projectStatus.description), { to: 'string' }));

            // snip24?
            if (projectStatus.snip24_info) {
                snip24Info = projectStatus.snip24_info;
                contributorNumberOfTokens = snip24Info.contributor_vesting_schedule.reduce((partialSum, e) => partialSum + BigInt(e.amount), BigInt(0));
                creatorNumberOfTokens = snip24Info.creator_vesting_schedule.reduce((partialSum, e) => partialSum + BigInt(e.amount), BigInt(0));
                totalInitialSupply = contributorNumberOfTokens + creatorNumberOfTokens;
            }

            if (projectStatus.pledged_message) {
                pledgedMessageFromPako = JSON.parse(pako.ungzip(atob(projectStatus.pledged_message), { to: 'string' }));
            }
            if (projectStatus.funded_message) {
                fundedMessageFromPako = JSON.parse(pako.ungzip(atob(projectStatus.funded_message), { to: 'string' }));
            }
            rewardMessagesFromPako = projectStatus.reward_messages.map( rm => {
                return {
                    threshold: parseFloat(rm.threshold) / 1000000,
                    message: JSON.parse(pako.ungzip(atob(rm.message), { to: 'string' }))
                };
            });
            // only one reward message at the moment
            if (rewardMessagesFromPako.length > 0) {
                rewardMessageFromPako = rewardMessagesFromPako[0];
            }
        }
    }

    loadProject();

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
        const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;

        if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
    	} else if (contributionValue === '' || contributionValue === '0') {
            toast.push("You must enter an amount to contribute");
        } else {
            try {
                const contribution = (Math.floor(parseFloat(contributionValue) * 1000000)).toString();
                await sscrt.send(scrtClient, projectContract.address, contribution);
				toast.push(`Successfully contributed ${contributionValue} sSCRT`);
                loadProject();
            } catch (err) {
                toast.push("Error sending contribution");
            }
        }
        contributionValue = "";
	}

    function handleNonNegativeInput(event) {
		if (event.target.valueAsNumber < 0) {
			contributionValue = event.target.value.substring(1);
		}
	}

    async function handleRefund() {
        const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;

        if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
    	} else {
            try {
                await projectContract.refund(scrtClient);
				toast.push(`Successfully refunded ${projectStatus.contribution} sSCRT`);
                loadProject();
            } catch (err) {
                toast.push("Error executing refund transaction");
            }
        }
        contributionValue = "";
    }

    async function handlePayOut() {
        const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;

        if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
    	} else {
            try {
                await projectContract.payOut(scrtClient);
				toast.push(`Congratulations your crowdfunding has been paid out!`);
                loadProject();
            } catch (err) {
                toast.push("Error executing pay out transaction");
            }
        }
        contributionValue = "";
    }

    async function handleComment() {
        const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;

        if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
    	} else if (commentValue === '') {
            toast.push("Comment is empty!")
        } else {
            try {
                await projectContract.comment(scrtClient, commentValue);
				toast.push(`Your comment has been added`);
                comments.push({ comment: commentValue, from_creator: projectStatus.creator === scrtClient.address });
                comments = comments;
            } catch (err) {
                toast.push("Error creating comment");
            }
        }
        commentValue = "";
    }

    async function handleSpam() {
        const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;

        if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
        } else {
            try {
                await projectContract.flag_spam(scrtClient, true);
				toast.push(`You have flagged this project as spam`);
                loadProject();
            } catch (err) {
                toast.push("Error flagging spam");
            }
        }
    }

    async function loadComments() {
        const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;

        if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
        } else if (!loadedAllComments) {
            const pageSize = 50;
            let newComments = await projectContract.queryComments(scrtClient, commentsLoaded, pageSize);
            if (newComments.comments.length > 0) {
                comments.push(...newComments.comments);
                comments = comments;
                commentsLoaded = commentsLoaded + 1;
            } else {
                loadedAllComments = true;
            }
        }
    }
</script>

{#if projectStatus}
    <div transition:scale|local={{ start: 0.7 }}>
        <Paper transition elevation={4}>
            <LayoutGrid>
                <ProjectPreviewCells bind:projectStatus bind:currentBlock bind:totalNum bind:goalNum />
                <Cell span={12}>
                    <InnerGrid>
                        <Cell span={12}><h3>Project Description</h3></Cell>
                        <Cell span={12}>
                            <Editor data={descriptionFromPako} editorId="descriptionReader" readOnly={true} />
                        </Cell>
                    {#if snip24Info}
                        <Cell span={12}>
                            <Paper elevation={4}>
                                <InnerGrid>
                                    <Cell span={12}><h3>SNIP-24 reward information</h3></Cell>
                                    {#if snip24Info.contract_address}
                                        <Cell span={12}>Contract: {snip24Info.contract_address}</Cell>
                                    {/if}
                                    <Cell span={12}>Name: {snip24Info.name}</Cell>
                                    <Cell span={12}>Symbol: {snip24Info.symbol}</Cell>
                                    <Cell span={12}>Decimals: {snip24Info.decimals}</Cell>
                                    <Cell span={12}>Total initial supply: {totalInitialSupply}</Cell>
                                    <Cell span={12}>Enable mint: {snip24Info.enable_mint ? "yes" : "no"}</Cell>
                                    <Cell span={12}>Tokens for contributors: {contributorNumberOfTokens}</Cell>
                                    <Cell span={12}><h4>Creator vesting schedule:</h4></Cell>
                                    {#each snip24Info.creator_vesting_schedule as e}
                                        <Cell span={6}>
                                            {e.block === 0 ? "Initial disbursement" : "Number of blocks: " + e.block.toString()}
                                        </Cell>
                                        <Cell span={6}>
                                            Amount: {e.amount}
                                        </Cell>
                                    {/each}            
                                </InnerGrid>
                            </Paper>
                        </Cell>
                    {/if}
                    {#if pledgedMessageFromPako}
                        <Cell span={12}><h3>Pledged message</h3></Cell>
                        <Cell span={12}>
                            <Editor data={pledgedMessageFromPako} editorId="pledgeMessageReader" readOnly={true} />
                        </Cell>
                    {/if}
                    {#if fundedMessageFromPako}
                        <Cell span={12}><h3>Funded message</h3></Cell>
                        <Cell span={12}>
                            <Editor data={fundedMessageFromPako} editorId="fundedMessageReader" readOnly={true} />
                        </Cell>
                    {/if}
                    {#if rewardMessageFromPako}
                        <Cell span={12}><h3>Reward message</h3></Cell>
                        <Cell span={12}>
                            <Editor data={rewardMessageFromPako.message} editorId="rewardMessageReader" readOnly={true} />
                        </Cell>
                    {/if}
                    </InnerGrid>
                </Cell>
                <Cell span={12}>
                    {#if !doneProject(projectStatus) && !isProjectCreator(projectStatus.creator)}
                        <div class="solo-demo-container solo-container-left">
                            <Paper class="solo-paper" elevation={6}>
                                <img src="sscrt.svg" alt="sscrt" style="color:white;"/>
                                <Input
                                    bind:value={contributionValue}
                                    placeholder="Amount you want to contribute"
                                    class="solo-input"
                                    type="number"
                                    style="font-size:20px;"
                                    on:input={handleNonNegativeInput}
                                />
                            </Paper>
                            <button class="button-beach-sm" on:click={handleContribute} >
                                <Label>Contribute</Label>
                            </button>
                        </div>
                    {/if}
                    <div class="solo-demo-container solo-container-left">
                        {#if projectStatus.contribution && projectStatus.contribution !== '0' && projectStatus.status !== "successful"}
                            <button class="button-beach-sm" on:click={handleRefund} >
                                <Label>Refund</Label>
                            </button>
                        {/if}
                        {#if isProjectCreator(projectStatus.creator) && readyForPayout(projectStatus)}
                            <button class="button-beach-sm" on:click={handlePayOut} >
                                <Label>Pay out</Label>
                            </button>
                        {/if}
                        <Paper class="solo-paper" elevation={6}>
                            <Input
                                bind:value={commentValue}
                                placeholder="Enter comment"
                                class="solo-input"
                                type="string"
                                style="font-size:14px;"
                            />
                        </Paper>
                        <button class="button-beach-sm" on:click={handleComment} >
                            <Label>Comment</Label>
                        </button>
                        <button class="button-beach-sm" on:click={handleSpam} >
                            <Label>Mark spam</Label>
                        </button>
                    </div>
                    <div class="solo-demo-container">
                        <h3>Comments</h3>
                        {#each comments as comment}
                            <p class={comment.from_creator ? 'creatorcomment' : ''}>{comment.comment}</p>
                        {/each}
                        <button class="textbtn" on:click={loadComments}>Load more...</button>
                    </div>
                </Cell>
            </LayoutGrid>
        </Paper>
    </div>
{/if}

<style lang="scss">	
	h4 {
		font-size: 14px;
        font-weight: 800;
	}

    p {
        word-wrap: break-word;
    }

    .edmargin {
		margin: 1rem 0 0 0;
	}

    .solo-demo-container {
        padding: 18px 10px;
        border: 1px solid rgba(255, 255, 255, 0.8);
    }

    .solo-demo-container-no-border {
        padding: 18px 10px;
    }
 
    .solo-container {
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .solo-container-left {
        display: flex;
        justify-content: start;
        align-items: center;
    }

    * :global(.solo-paper) {
        display: flex;
        align-items: center;
        flex-grow: 1;
        max-width: 430px;
        margin: 0 12px;
        padding: 0 12px;
        height: 48px;
    }

    * :global(.solo-paper > *) {
        display: inline-block;
        margin: 0 12px;
    }

    * :global(.solo-input) {
        flex-grow: 1;
        color: white;
        //color: var(--mdc-theme-on-surface, #000);
    }

    * :global(.solo-input::placeholder) {
        //color: var(--mdc-theme-on-surface, #000);
        color: white;
        opacity: 0.7;
    }

    .textbtn {
        color: white;
        border: none;
        background-color: inherit;
        padding: 14px 28px;
        font-size: 16px;
        cursor: pointer;
        display: inline-block;
    }

    .creatorcomment {
        color: yellow;
    }
</style>