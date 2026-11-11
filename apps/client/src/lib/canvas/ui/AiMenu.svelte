<script lang="ts">
	import { Button, Field, Icon, Menu, Modal, PlanLimit, RadioGroup, Textarea, toast } from '@facile/muse';
	import type { PlanLimitError } from '@facile/muse';
	import { aiError, aiPlanLimit, clusterShapes, generateShapes, type DrawKind } from '$lib/api/ai';
	import type { Editor } from '../editor.svelte';
	import { toolIcons } from '../icons';

	let {
		board,
		editor,
		busy = $bindable(false),
		onsummarize,
		limit = $bindable(null)
	}: {
		board: string;
		editor: Editor;
		busy?: boolean;
		onsummarize: () => void;
		limit?: PlanLimitError | null;
	} = $props();

	let composing = $state(false);
	let prompt = $state('');
	let kind = $state<DrawKind>('mindmap');

	const KINDS = [
		{ value: 'mindmap', label: 'Carte mentale' },
		{ value: 'flowchart', label: 'Logigramme' },
		{ value: 'stickies', label: 'Post-it' }
	];

	const stickies = $derived(editor.selection.filter((id) => editor.scene.get(id)?.type === 'sticky'));

	async function run<T>(action: () => Promise<T>, done: (result: T) => string) {
		busy = true;
		try {
			toast.success(done(await action()));
		} catch (error) {
			limit = aiPlanLimit(error);
			if (!limit) toast.danger(aiError(error));
		} finally {
			busy = false;
		}
	}

	function cluster() {
		const ids = stickies;
		void run(
			() => clusterShapes(board, ids),
			(frames) => `${ids.length} post-it rangés en ${frames.length} thèmes.`
		);
	}

	function generate(event: SubmitEvent) {
		event.preventDefault();
		const text = prompt.trim();
		if (!text) return;
		composing = false;
		void run(
			() => generateShapes(board, text, kind),
			(created) => {
				prompt = '';
				return `${created.length} formes ajoutées à droite du tableau.`;
			}
		);
	}

	const items = $derived([
		{ label: 'Résumer le tableau', onselect: onsummarize, disabled: busy },
		{
			label: stickies.length >= 2 ? `Regrouper ${stickies.length} post-it par thème` : 'Regrouper la sélection (2 post-it ou plus)',
			onselect: cluster,
			disabled: busy || stickies.length < 2
		},
		{ label: 'Générer…', onselect: () => (composing = true), disabled: busy }
	]);
</script>

<Menu label="IA" {items} align="end" data-testid="ai-menu">
	{#snippet trigger()}<Icon icon={toolIcons.ai} size={20} />{/snippet}
</Menu>

<Modal bind:open={composing} title="Générer avec l’IA" size="sm">
	<form id="ai-generate" class="flex flex-col gap-4" onsubmit={generate}>
		<Field label="Que voulez-vous dessiner ?">
			<Textarea bind:value={prompt} rows={4} maxlength={2000} placeholder="Le lancement de la rentrée, étape par étape" required />
		</Field>
		<Field label="Forme">
			<RadioGroup options={KINDS} bind:value={kind} orientation="horizontal" />
		</Field>
	</form>
	{#snippet footer()}
		<Button variant="ghost" onclick={() => (composing = false)}>Annuler</Button>
		<Button type="submit" form="ai-generate" disabled={!prompt.trim()}>Générer</Button>
	{/snippet}
</Modal>

<PlanLimit error={limit} variant="dialog" onClose={() => (limit = null)} />
