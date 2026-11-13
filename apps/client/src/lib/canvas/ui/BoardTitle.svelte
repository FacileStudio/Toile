<script lang="ts">
	import { tick } from 'svelte';
	import { Tooltip, toast } from '@facile/muse';
	import { boardApi } from '$lib/board/api';

	let { name, board }: { name: string; board: string | null } = $props();

	let shown = $state('');
	let draft = $state('');
	let editing = $state(false);
	let field = $state<HTMLInputElement>();

	$effect(() => {
		shown = name;
		draft = name;
	});

	async function save() {
		if (!editing) return;
		editing = false;
		const trimmed = draft.trim();
		if (!trimmed || trimmed === shown || !board) return;
		try {
			await boardApi.rename(board, trimmed);
			shown = trimmed;
		} catch {
			draft = shown;
			toast.danger('Renommage impossible');
		}
	}

	function start() {
		editing = true;
		void tick().then(() => field?.select());
	}
</script>

{#if editing}
	<input
		bind:this={field}
		bind:value={draft}
		class="max-w-[40vw] rounded-fc-xs bg-fc-surface px-2 py-0.5 text-fc-sm font-semibold text-fc-fg focus:outline-2 focus:outline-fc-ring"
		aria-label="Nom du tableau"
		onblur={save}
		onkeydown={(e) => {
			if (e.key === 'Enter') void save();
			else if (e.key === 'Escape') (draft = shown), (editing = false);
		}}
	/>
{:else if board}
	<Tooltip text="Renommer le tableau">
		<button
			type="button"
			class="max-w-[40vw] cursor-pointer truncate rounded-fc-xs px-2 py-0.5 text-left text-fc-sm font-semibold text-fc-fg hover:bg-fc-surface/60 focus-visible:outline-2 focus-visible:outline-fc-ring"
			aria-label="Renommer le tableau {shown}"
			onclick={start}
		>
			{shown}
		</button>
	</Tooltip>
{:else}
	<h1 class="max-w-[40vw] truncate px-2 py-0.5 text-fc-sm font-semibold text-fc-fg">{shown}</h1>
{/if}
