<script lang="ts">
	import { keplrStore } from "./stores/keplr";
	import { get } from 'svelte/store';
    import { fade } from 'svelte/transition';
    import { push } from 'svelte-spa-router';
    import { toast } from '@zerodevx/svelte-toast';
	import { fromUtf8 } from "secretjs";
	import { MsgExecuteContractResponse } from "secretjs/dist/protobuf_stuff/secret/compute/v1beta1/msg";

	import { CreateResponse } from "./lib/contract";
    import { PLATFORM_CODE_HASH, SSCRT_CODE_HASH, PLATFORM_CONTRACT, SSCRT_CONTRACT } from './lib/contracts';
    import { allCategories } from './lib/categories';

    import Paper from '@smui/paper';
    import { Input } from '@smui/textfield';
	import Textfield from '@smui/textfield';
  	import CharacterCounter from '@smui/textfield/character-counter/index';
	import { Label } from '@smui/button';

    import MultiSelect from 'svelte-multiselect';
	import Editor from './Editor.svelte';

	const platform: PlatformContractInstance = new PlatformContractInstance("platform", PLATFORM_CODE_HASH, PLATFORM_CONTRACT);
	const denominations = [
		{ id: 0, text: "sSCRT", img: "sscrt.svg", alt: "sscrt"}
	];

	let title: string = '';
	let subtitle: string = '';
	let categories: string[] = [];
	let description: string = '';
	let pledged_message: string = '';
	let funded_message: string = '';
    let deadline: number = 14;
	let goal: string = '';
	let denom = 0;
	let result: CreateResponse;

    $: invalidProject = title === '' || description === '' || goal === '' || parseFloat(goal) <= 0 || !deadline;
	$: categoryIndexes = categories.map( (category) => {
		return allCategories.indexOf(category);
	});
	import pako from "pako";
    import { PlatformContractInstance, PlatformCreateMsg } from './lib/contracts';
    import { daysInBlocks, entropy, getBlock } from "./lib/utils";
	$: pakoDescription = btoa(pako.gzip(description, {to: 'string'}));
	$: pakoPledgedMessage = btoa(pako.gzip(pledged_message, {to: 'string'}));
	$: pakoFundedMessage = btoa(pako.gzip(funded_message, {to: 'string'}));

    function clearFields() {
		title = '';
		subtitle = '';
		categories = [];
		description = '';
		pledged_message = '';
		funded_message = '';
		goal = '';
		deadline = null;
		denom = 0;
		result = null;
	}

    async function handleStartFundraising() {
		const keplr = get(keplrStore);
    	const {keplrEnabled, scrtAuthorized, scrtClient} = keplr;
    	if (!keplrEnabled || !scrtAuthorized) {
        	toast.push("Keplr not enabled");
    	} else {
			const goalUScrt = (Math.floor(parseFloat(goal) * 1000000)).toString();
			const currentBlock = await getBlock(scrtClient);
			const deadlineBlock = daysInBlocks(deadline) + currentBlock;
			const platformCreateMsg: PlatformCreateMsg = {
				title,
				subtitle,
				description: pakoDescription,
				pledged_message: pakoPledgedMessage,
				funded_message: pakoFundedMessage,
				reward_messages: [], //TODO
				goal: goalUScrt,
				deadline: deadlineBlock,
				categories: categoryIndexes,
				snip20_contract: SSCRT_CONTRACT,
				snip20_hash: SSCRT_CODE_HASH,
				entropy: entropy(),
			};
			try {
				const tx = await platform.create(scrtClient, platformCreateMsg, 500_000);
				result = JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(tx.data[0]).data));
				if (result.create && result.create.status === "success") {
					clearFields();
					push('/projects/0');
					toast.push("Fundraising project has been created");
				} else {
					toast.push("Error creating fundraising project");
				}
			} catch (error) {
        		toast.push(error.toString());
    		}
		}
	}

	function handleGoalInput(event) {
		if (event.target.valueAsNumber < 0) {
			goal = event.target.value.substring(1);
		}
	}
</script>

<svelte:head>
	<title>Mage Hands: Create</title>
</svelte:head>

<section in:fade="{{duration: 500}}" class="newproj">
	<div class="lgmargin">
    	<h1>Create a New Fundraising Project</h1>
	</div>

	<div class="margins">
        <div class="solo-demo-container solo-container">
			<button on:click={() => handleStartFundraising()} >
				<Label>BASICS</Label>
			</button>
			<button on:click={() => handleStartFundraising()} >
				<Label>REWARDS</Label>
			</button>
			<button on:click={() => handleStartFundraising()} >
				<Label>STORY</Label>
			</button>
			<button on:click={() => handleStartFundraising()} >
				<Label>PEOPLE</Label>
			</button>
          </div>
    </div>

	<div class="margins">
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
	</div>

	<div>
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
	</div>

     <div class="solo-demo-container-no-border solo-container">
        <MultiSelect
            bind:selected={categories} 
            options={allCategories} 
            maxSelect={3}
            placeholder="Select categories..."
            --sms-options-bg="var(--pure-white, white)"
            --sms-token-bg="var(--pure-white)"
            --sms-text-color="var(--secondary-color)"
            --sms-li-selected-color="var(--secondary-color)"
            --sms-token-padding="0.5ex 0 0.5ex 1ex"
        />
    </div>

	<div class="lgmargin">
		<div class="solo-demo-container-no-border solo-container">
			Describe your project. Paste image or video urls to embed.
		</div>
		<Editor bind:data={description} editorId="descriptionEditor"/>
	</div>

	<div class="lgmargin">
		<div class="solo-demo-container-no-border solo-container">
			Add an optional message for contributors when they pledge a contribution.
		</div>
		<Editor bind:data={pledged_message} editorId="pledgedMessageEditor"/>
	</div>

	<div class="lgmargin">
		<div class="solo-demo-container-no-border solo-container">
			Add an optional message for contributors only visible after successful fundraising and payout.
		</div>
		<Editor bind:data={funded_message} editorId="fundedMessageEditor"/>
	</div>
	
    <div class="margins">
        <div class="solo-demo-container solo-container">
            <Paper class="solo-paper" elevation={6}>
                <img src="sscrt.svg" alt="sscrt" style="color:white;"/>
                <Input
                    bind:value={goal}
                    placeholder="Minimum amount you want to raise"
                    class="solo-input"
                    type="number"
                    style="font-size:20px;"
					on:input={handleGoalInput}
                />
            </Paper>
          </div>
    </div>

    <div class="lgmargin">
        <div class="solo-demo-container-no-border solo-container">
            <label>
	        <input type=radio bind:group={deadline} name="deadline" value={14} />
	            14 days
            </label>

            <label>
	        <input type=radio bind:group={deadline} name="deadline" value={30}>
	            30 days
            </label>

            <label>
	        <input type=radio bind:group={deadline} name="deadline" value={60}>
	            60 days
            </label>
        </div>
    </div>

    <div class="margins">
        <div class="solo-demo-container-no-border solo-container">
		    <button on:click={() => handleStartFundraising()} disabled={invalidProject} class="button-beach">
			    <Label>Start Fundraising</Label>
		    </button>
        </div>
    </div>
</section>

<style lang="scss">
    @import url("https://fonts.googleapis.com/css?family=Raleway:500");

    .button-beach {
        -webkit-appearance: none;
        background: -webkit-gradient(to right, #064a45 0%, #fceeb5 50%, #ee786e 100%);
        background: linear-gradient(to right, #064a45 0%, #fceeb5 50%, #ee786e 100%);
        background-size: 500%;
        border: none;
        border-radius: 5rem;
        box-shadow: 0 0.5rem 1rem rgba(0, 0, 0, 0.15);
        color: #fff;
        cursor: pointer;
        font: 1.5em Raleway, sans-serif;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
        height: 5rem;
        letter-spacing: 0.05em;
        outline: none;
        -webkit-tap-highlight-color: transparent;
        -webkit-user-select: none;
        -moz-user-select: none;
        -ms-user-select: none;
        user-select: none;
        width: 20rem;
    }

    .button-beach:hover {
        animation-name: gradient;
        -webkit-animation-name: gradient;
        animation-duration: 2s;
        -webkit-animation-duration: s;
        animation-iteration-count: 1;
        -webkit-animation-iteration-count: 1;
        animation-fill-mode: forwards;
        -webkit-animation-fill-mode: forwards;
    }

    @keyframes gradient {
        0% {
            background-position: 0% 50%;
        }
        100% {
            background-position: 100%;
        }
    }

	button {
		background: var(--accent-color-dark);
		color: #fff;
		border: 0;
		padding: 18px 30px;
		font-size: 1.2em;
		border-radius: 6px;
		cursor: pointer;
	}

	.newproj {
		width: 100%;
		max-width: var(--column-width);
		margin: var(--column-margin-top) auto 0 auto;
		line-height: 1;
	}

	.margins {
		margin: 0 0 1rem 0;
	}

	.smmargin {
		margin: 0 0 0.5rem 0;
	}

	.lgmargin {
		margin: 0 0 2.5rem 0;
	}

	.submit {
		margin: auto;
	}

	* :global(.multiselect ul.tokens > li) {
  		/* the blue tags representing selected options with remove buttons inside the input */
		color: var(--secondary-color);
	}

	* :global(.multiselect ul.options li) {
  		/* dropdown options */
		color: var(--secondary-color);
	}

	* :global(.shaped-outlined),
	* :global(.shaped-outlined .mdc-select__anchor) {
    	border-radius: 28px;
  	}
  	* :global(.shaped-outlined .mdc-text-field__input) {
    	padding-left: 32px;
    	padding-right: 0;
  	}
  	* :global(.shaped-outlined
      	.mdc-notched-outline
      	.mdc-notched-outline__leading) {
		border-radius: 28px 0 0 28px;
		width: 28px;
  	}
  	* :global(.shaped-outlined
      	.mdc-notched-outline
    	.mdc-notched-outline__trailing) {
		border-radius: 0 28px 28px 0;
  	}
	* :global(.shaped-outlined .mdc-notched-outline .mdc-notched-outline__notch) {
    	max-width: calc(100% - 28px * 2);
  	}
  	* :global(.shaped-outlined.mdc-select--with-leading-icon
    	.mdc-notched-outline:not(.mdc-notched-outline--notched)
    	.mdc-floating-label) {
    	left: 16px;
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
</style>
