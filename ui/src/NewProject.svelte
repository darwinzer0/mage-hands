<script lang="ts">
	import { keplrStore } from "./stores/keplr";
	import { get } from 'svelte/store';
    import { fade } from 'svelte/transition';
    import { push } from 'svelte-spa-router';
    import { toast } from '@zerodevx/svelte-toast';
	import { fromUtf8 } from "secretjs";
	import { MsgExecuteContractResponse } from "secretjs/dist/protobuf_stuff/secret/compute/v1beta1/msg";

	import { CreateResponse } from "./lib/contract";
    import { PLATFORM_CODE_HASH, SSCRT_CODE_HASH, PLATFORM_CONTRACT, SSCRT_CONTRACT, ProjectRewardMessage } from './lib/contracts';
    import { allCategories } from './lib/categories';

    import Paper from '@smui/paper';
    import { Input } from '@smui/textfield';
	import Textfield from '@smui/textfield';
  	import CharacterCounter from '@smui/textfield/character-counter';
	import LayoutGrid, { Cell, InnerGrid } from '@smui/layout-grid';
	import { Label } from '@smui/button';
    import FormField from '@smui/form-field';
	import Switch from "@smui/switch";
    import MultiSelect from 'svelte-multiselect';
	import Editor from './Editor.svelte';
	import RewardEditor from "./RewardEditor.svelte";
	import pako from "pako";
    import { PlatformContractInstance, PlatformCreateMsg } from './lib/contracts';
    import { daysInBlocks, entropy, getBlock } from "./lib/utils";
	import Resizer from "react-image-file-resizer"; 

	const platform: PlatformContractInstance = new PlatformContractInstance("platform", PLATFORM_CODE_HASH, PLATFORM_CONTRACT);
	const denominations = [
		{ id: 0, text: "sSCRT", img: "sscrt.svg", alt: "sscrt"}
	];

	type RewardMessage = {
		message: string,
		threshold: number,
	};

	let title: string = '';
	let subtitle: string = '';
	let categories: string[] = [];
	let description: string = '';
	let rawlog: string = '';

	const resize = Resizer.imageFileResizer;
	let rawInput; 
	let cover_img: string = '';

	let pledged_message: string = '';
	let funded_message: string = '';
	let reward_messages: RewardMessage[] = [];
    let deadline: number = 14;
	let goal: string = '';

	// snip24
	type VestingEvent = {
		days: number;
		percentage: number;
	};

	let snip24Enabled: boolean = false;

	let snip24Name: string = '';
	let snip24Symbol: string = '';
	let snip24Decimals: number = 6;
	let snip24Admin: string = '';
	let enablePublicTokenSupply: boolean = true;
	let enableDeposit: boolean = false;
	let enableRedeem: boolean = false;
	let enableMint: boolean = false;
	let enableBurn: boolean = false;

	let numberOfTokens: string = '';

	let contributorPctTokens: string = '';
	let contributorVestingSchedule: VestingEvent[] = [];

	let snip24MinContribution: string = '';
	let snip24MaxContribution: string = '';
	let snip24;

	let result: CreateResponse;

	const subscreens = ["Basics", "Details", "Rewards", "Tokens", "Upload"];
	let subscreen: string = subscreens[0];
	const deadlineOptions = [14, 30, 60];

    $: invalidProject = title === '' || description === '' || goal === '' || parseFloat(goal) <= 0 || !deadline || categories.length === 0;
	$: categoryIndexes = categories.map( (category) => {
		return allCategories.indexOf(category);
	});

    function clearFields() {
		title = '';
		subtitle = '';
		categories = [];
		description = '';
		pledged_message = '';
		funded_message = '';
		reward_messages = [];
		goal = '';
		deadline = null;
		result = null;
	}

	async function handleSubScreenButton(sub: string) {
		subscreen = sub;
		//toast.push(sub);
	}

    async function handleStartFundraising() {
		const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;

		if (invalidProject) {
			toast.push("Missing required information");
		} else if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
    	} else {
			const goalUScrt = (Math.floor(parseFloat(goal) * 1000000)).toString();
			const currentBlock = await getBlock(scrtClient);
			const deadlineBlock = daysInBlocks(deadline) + currentBlock;

			const pakoDescription = btoa(pako.gzip(description, {to: 'string'}));
			const pakoPledgedMessage = btoa(pako.gzip(pledged_message, {to: 'string'}));
			const pakoFundedMessage = btoa(pako.gzip(funded_message, {to: 'string'}));
			const pakoRewardMessages: ProjectRewardMessage[] = reward_messages.map(m => {
				const pakoRewardMessageMessage = btoa(pako.gzip(m.message, {to: 'string'}));
				const pakoRewardMessage: ProjectRewardMessage = {
					message: pakoRewardMessageMessage,
					threshold: (Math.floor(m.threshold * 1000000)).toString()
				};
				return pakoRewardMessage;
			});

			const platformCreateMsg: PlatformCreateMsg = {
				title,
				subtitle,
				description: pakoDescription,
				cover_img, 
				pledged_message: pakoPledgedMessage,
				funded_message: pakoFundedMessage,
				reward_messages: pakoRewardMessages,
				goal: goalUScrt,
				deadline: deadlineBlock,
				categories: categoryIndexes,
				snip20_contract: SSCRT_CONTRACT,
				snip20_hash: SSCRT_CODE_HASH,
				entropy: entropy(),
			};

			try {				
				const tx = await platform.create(scrtClient, platformCreateMsg);
				rawlog = tx.rawLog;
				result = JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(tx.data[0]).data));
				//toast.push(JSON.stringify(result));
				if (result.create && result.create.status === "success") {
					clearFields();
					push('/projects/0');
					toast.push("Fundraising project has been created");
				} else {
					toast.push("Error creating fundraising project");
				}
			} catch (error) {
				//toast.push(error.toString());
        		toast.push("Error creating fundraising project");
    		}
		}
	}

	function handleNonNegativeInput(event) {
		if (event.target.valueAsNumber < 0) {
			goal = event.target.value.substring(1);
		}
	}

	function handleSymbolInput(event) {
		if (event.target.value) {
			snip24Symbol = event.target.value.toUpperCase();
		}
	}

	function handleDecimalsInput(event) {
		if (event.target.valueAsNumber < 0) {
			snip24Decimals = 0;
		}
		if (event.target.valueAsNumber > 18) {
			snip24Decimals = 18;
		}
	}

	function handleAddRewardMessage() {
		reward_messages.push({ message: "", threshold: 0 });
		reward_messages = reward_messages;
	}

	const resizeImage = (img): Promise<string | Blob | File | ProgressEvent<FileReader>> => {
        return new Promise((resolve, reject) => {
           resize(img, 250, 250, "WEBP", 100, 0, uri => resolve(uri), 'base64');
        });
    };

	const onFileChange = async () => {
		if (rawInput && rawInput.files.length > 0) {
			const file = rawInput.files[0];
			cover_img = (await resizeImage(file)) as string;
		}
	};
</script>

<svelte:head>
	<title>Mage Hands: Create</title>
</svelte:head>

<section in:fade="{{duration: 500}}" class="newproj">
	<div class="margins">
    	<h1>Create a New Fundraising Project</h1>
	</div>

	<div class="margins">
        <div class="solo-demo-container solo-container">
			{#each subscreens as sub}
				<button class={subscreen === sub ? "button-beach-sm-selected" : "button-beach-sm"} on:click={() => handleSubScreenButton(sub)} >
					<Label>{sub}</Label>
				</button>
			{/each}
          </div>
    </div>

	<Paper transition elevation={4}>
		<div class={subscreen === subscreens[0] ? "" : "hidden-div"}>
			<LayoutGrid>
				<Cell span={3}>
					<p>
						The basic information about your project that will show up on the project list page.
					</p>
				</Cell>
				<Cell span={9}>
					<InnerGrid>
						<Cell span={12}>
							<Textfield
								style="width: 100%;"
								helperLine$style="width: 100%;"
								variant="outlined"
								bind:value={title}
								label="Title"
								input$maxlength={100}
								input$style="font-size:28px;"
							>
								<CharacterCounter slot="helper">0 / 100</CharacterCounter>
							</Textfield>
						</Cell>
						<Cell span={12}>
							<Textfield
								style="width: 100%;"
								helperLine$style="width: 100%;"
								variant="outlined"
								bind:value={subtitle}
								label="Subtitle"
								input$maxlength={100}
								input$style="font-size:24px;"
							>
								<CharacterCounter slot="helper">0 / 100</CharacterCounter>
							</Textfield>
						</Cell>
						<Cell span={3}>
							<label for="imginput"><h2>Cover</h2></label>
						</Cell>
						<Cell span={9}>
							<input
								id="imginput"
								bind:this={rawInput}
								on:change={onFileChange}
								  type="file"	
							/>
							{#if cover_img != ''}
								<img src={cover_img} class="edmargin" />
							{/if}
						</Cell>
						<Cell span={12}>
							<MultiSelect
								bind:selected={categories} 
								options={allCategories} 
								maxSelect={3}
								placeholder="Categories..."
								--sms-options-bg="var(--pure-white, white)"
								--sms-token-bg="var(--pure-white)"
								--sms-text-color="var(--secondary-color)"
								--sms-li-selected-color="var(--secondary-color)"
								--sms-li-selected-bg="var(--pure-white, white)"
								--sms-token-padding="0.5ex 0 0.5ex 1ex"
								--sms-font-size="24px"
								--sms-max-width="100%"
								--sms-padding="1pt 3pt"
								--sms-bg="var(--pure-white, white)"
							/>
						</Cell>
						<Cell span={12}>
							<div class="solo-demo-container solo-container">
								<Paper class="solo-paper" elevation={6}>
									<img src="sscrt.svg" alt="sscrt" style="color:white;"/>
									<Input
										bind:value={goal}
										placeholder="Amount you want to raise"
										class="solo-input"
										type="number"
										style="font-size:20px;"
										on:input={handleNonNegativeInput}
									/>
								</Paper>
							</div>
						</Cell>
					</InnerGrid>
				</Cell>
			</LayoutGrid>
		</div>
		<div class={subscreen === subscreens[1] ? "" : "hidden-div"}>
			<LayoutGrid>
				<Cell span={3}>
					<p>
						Describe your project here. You can embed images or videos on the web by copying the URL into the edit box.
					</p>
					<p>
						The more specific information you can add here about your goals for project and vision, 
						the more likely you are to be successful in your fundraising goal. 
					</p>
					<p>
						Including visuals or links to external sites with information about your project 
						is a great way for people to learn more about your project. 
						Also, add as much information as you can about your team and background, and use social media platforms 
						to build interest in advance of launching your campaign.
					</p>
				</Cell>
				<Cell span={9}>
					<InnerGrid>
						<Cell span={12}>
							<h2>Project description</h2>
						</Cell>
						<Cell span={12}>
							<Editor bind:outputData={description} editorId="descriptionEditor"/>
						</Cell>
					</InnerGrid>
				</Cell>
			</LayoutGrid>
		</div>
		<div class={subscreen === subscreens[2] ? "" : "hidden-div"}>
			<LayoutGrid>
				<Cell span={3}>
					<p>
						Add an optional message for all contributors when they pledge a contribution. This is a good place to add a simple thank you message.
					</p>
				</Cell>
				<Cell span={9}>
					<InnerGrid>
						<Cell span={12}>
							<h2>Pledged message</h2>
						</Cell>
						<Cell span={12}>
							<Editor bind:outputData={pledged_message} editorId="pledgedMessageEditor"/>
						</Cell>
					</InnerGrid>
				</Cell>
				<Cell span={3}>
					<p>
						Add an optional message for all contributors that is only visible after successful fundraising. Some things you could put here include whitelist codes or a link to an encrypted file and the decryption key.
					</p>
				</Cell>
				<Cell span={9}>
					<InnerGrid>
						<Cell span={12}>
							<h2>Funded message</h2>
						</Cell>
						<Cell span={12}>
							<Editor bind:outputData={funded_message} editorId="fundedMessageEditor"/>
						</Cell>
					</InnerGrid>
				</Cell>
				<Cell span={3}>
					<p>
						Add special messages that are only visible to contributors who give above a threshold. You can reward big backers with whitelist codes to exclusive NFT drops or encrypted files.
					</p>
				</Cell>
				<Cell span={9}>
					<InnerGrid>
						<Cell span={12}>
							<h2>Reward messages</h2>
						</Cell>
						{#each reward_messages as reward_message, i}
							<Cell span={12}>
								<RewardEditor bind:outputData={reward_messages} messageIdx={i} editorId={"rewardMessageId"+i} />
								<div class="solo-demo-container solo-container">
									Threshold:
									<Paper class="solo-paper" elevation={6}>
										<img src="sscrt.svg" alt="sscrt" style="color:white;"/>
										<Input
											bind:value={reward_messages[i].threshold}
											placeholder="Reward threshold"
											class="solo-input"
											type="number"
											style="font-size:16px;"
											on:input={handleNonNegativeInput}
										/>
									</Paper>
								</div>
							</Cell>
						{/each}
						<Cell span={12}>
							<button class="button-beach-sm" on:click={() => handleAddRewardMessage()} >
								<Label>Message+</Label>
							</button>
						</Cell>
					</InnerGrid>
				</Cell>
			</LayoutGrid>
		</div>
		<div class={subscreen === subscreens[3] ? "" : "hidden-div"}>
			<LayoutGrid>
				<Cell span={3}>
					<p>
						Select here if you want to issue a SNIP-24 token for your project.
					</p>
				</Cell>
				<Cell span={9}>
					<FormField>
						<Switch bind:checked={snip24Enabled} />
						<span slot="label">Add SNIP-24 Reward</span>
					</FormField>
				</Cell>
				{#if snip24Enabled}
					<Cell span={6}>
						<Paper class="lightpaper" elevation={6}>
							<h2>Details</h2>
							<InnerGrid>
								<Cell span={12}>
									<Textfield
										style="width: 100%;"
										helperLine$style="width: 100%;"
										variant="outlined"
										bind:value={snip24Name}
										label="Name"
										input$minlength={3}
										input$maxlength={30}
										input$style="font-size:20px;"
									>
										<CharacterCounter slot="helper">0 / 30</CharacterCounter>
									</Textfield>
								</Cell>
								<Cell span={12}>
									<Textfield
										style="width: 100%;"
										helperLine$style="width: 100%;"
										variant="outlined"
										bind:value={snip24Symbol}
										label="Symbol"
										input$maxlength={12}
										input$style="font-size:20px;"
										on:input={handleSymbolInput}
									>
										<CharacterCounter slot="helper">0 / 6</CharacterCounter>
									</Textfield>
								</Cell>
								<Cell span={12}>
									<Textfield
										style="width: 100%;"
										helperLine$style="width: 100%;"
										variant="outlined"
										bind:value={snip24Decimals}
										label="Decimals"
										input$maxlength={2}
										input$style="font-size:20px;"
										type="number"
										on:input={handleDecimalsInput}
									/>
								</Cell>
							</InnerGrid>
						</Paper>
					</Cell>
					<Cell span={6}>
						<Paper elevation={6}>
							<h2>Token configuration</h2>
							<InnerGrid>
								<Cell span={12}>
									<FormField>
										<Switch bind:checked={enablePublicTokenSupply} />
										<span slot="label">Enable public token supply</span>
									</FormField>
								</Cell>
								<Cell span={12}>
									<FormField>
										<Switch bind:checked={enableDeposit} />
										<span slot="label">Enable deposit</span>
									</FormField>
								</Cell>
								<Cell span={12}>
									<FormField>
										<Switch bind:checked={enableRedeem} />
										<span slot="label">Enable redeem</span>
									</FormField>
								</Cell>
								<Cell span={12}>
									<FormField>
										<Switch bind:checked={enableMint} />
										<span slot="label">Enable mint</span>
									</FormField>
								</Cell>
								<Cell span={12}>
									<FormField>
										<Switch bind:checked={enableBurn} />
										<span slot="label">Enable burn</span>
									</FormField>
								</Cell>
							</InnerGrid>
						</Paper>
					</Cell>
					<Cell span={6}>
						<Paper elevation={6}>
							<h2>Token disbursement</h2>
						</Paper>
					</Cell>
				{/if}
			</LayoutGrid>
		</div>
		<div class={subscreen === subscreens[4] ? "" : "hidden-div"}>
			<LayoutGrid>
				<Cell span={3}>
					<p>
						Choose the number of days you want to be fundraising your project. Note the actual deadline is a block height estimated based on an average of 6 seconds per block on Secret Network.
					</p>
					<p>
						Every project must have a title, at least one category, a description, and a goal.
					</p>
				</Cell>
				<Cell span={9}>
					<div class="lgmargin">
						<div class="solo-demo-container-no-border solo-container">
							{#each deadlineOptions as deadlineOption}
								<div class="radio-days">
									<label><input type=radio bind:group={deadline} name="deadline" value={deadlineOption} /> {deadlineOption} days</label>
								</div>
							{/each}
						</div>
					</div>
		
					<div class="margins">
						<div class="solo-demo-container-no-border solo-container">
							<button on:click={() => handleStartFundraising()} class="button-beach">
								<Label>Start Fundraising</Label>
							</button>
						</div>
					</div>
				</Cell>
			</LayoutGrid>
		</div>
	</Paper>
	{rawlog}
</section>

<style lang="scss">
    /* @import url("https://fonts.googleapis.com/css?family=Raleway:500"); */

/*	button {
		background: var(--accent-color-dark);
		color: #fff;
		border: 0;
		padding: 18px 30px;
		font-size: 1.2em;
		border-radius: 6px;
		cursor: pointer;
	}
*/
	.newproj {
		width: 100%;
		max-width: var(--column-width);
		margin: 0 auto 0 auto;
		line-height: 1;
	}

	.file-input {
		font-size: 24px;
	}

	.margins {
		margin: 0 0 1rem 0;
	}

	.edmargin {
		margin: 1rem 0 0 0;
	}

	.lgmargin {
		margin: 0 0 2.5rem 0;
	}

	.slider-select {
		background-color: var(--primary-color);
	}

	.lightpaper {
		color: var(--tertiary-color);
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

	.radio-days {
    	margin: 0 2em;
		font-size: 24px;
  	}

	* :global(.smui-paper) {
		background-color: #ffffff08;
	}

	.hidden-div {
		display: none;
	}
</style>
