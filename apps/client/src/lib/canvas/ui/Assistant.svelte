<script lang="ts">
	import { Blob, thinking } from '@facile/blob';
	import '@facile/blob/motion.css';
	import { Button, Icon, IconButton, PlanLimit, Textarea, Tooltip, icons, toast } from '@facile/muse';
	import type { PlanLimitError } from '@facile/muse';
	import { aiError, aiPlanLimit, clusterShapes, generateShapes, summarizeBoard, type DrawKind } from '$lib/api/ai';
	import { blocks } from '$lib/markdown';
	import { blobColor } from '../blob';
	import type { Editor } from '../editor.svelte';
	import { union } from '../geometry';
	import { showBox } from '../view';

	let { board, editor, name, onclose }: { board: string; editor: Editor; name: string; onclose: () => void } =
		$props();

	const KINDS: { value: DrawKind; label: string; hint: string }[] = [
		{ value: 'stickies', label: 'Post-it', hint: 'Des idées en post-it' },
		{ value: 'mindmap', label: 'Carte mentale', hint: 'Un sujet et ses branches' },
		{ value: 'flowchart', label: 'Logigramme', hint: 'Des étapes et des décisions' }
	];
	const heading = { 2: 'text-fc-md', 3: 'text-fc-sm', 4: 'text-fc-sm' } as const;

	let prompt = $state('');
	let kind = $state<DrawKind>('stickies');
	let busy = $state(false);
	let summary = $state<string | null>(null);
	let limit = $state<PlanLimitError | null>(null);
	let pending = $state<string[]>([]);
	let panel = $state<HTMLElement>();
	const chips: HTMLButtonElement[] = [];

	const stickies = $derived(editor.selection.filter((id) => editor.scene.get(id)?.type === 'sticky'));
	const content = $derived(blocks(summary ?? ''));

	$effect(() => panel?.querySelector('textarea')?.focus());

	$effect(() => {
		void editor.version;
		if (pending.length && pending.every((id) => editor.scene.shapes.has(id))) {
			editor.select(pending);
			pending = [];
		}
	});

	async function run<T>(action: () => Promise<T>, done: (result: T) => void) {
		busy = true;
		try {
			done(await action());
		} catch (error) {
			limit = aiPlanLimit(error);
			if (!limit) toast.danger(aiError(error));
		} finally {
			busy = false;
		}
	}

	function generate(event?: Event) {
		event?.preventDefault();
		const text = prompt.trim();
		if (!text || busy) return;
		void run(
			() => generateShapes(board, text, kind, blobColor(name)),
			(created) => {
				prompt = '';
				pending = created.map((shape) => shape.id);
				const box = union(created);
				if (box) showBox(editor, box);
				toast.success(`${name} a ajouté ${created.length} formes au tableau.`);
			}
		);
	}

	function summarize() {
		void run(() => summarizeBoard(board), (markdown) => (summary = markdown));
	}

	function cluster() {
		const ids = stickies;
		void run(
			() => clusterShapes(board, ids),
			(frames) => toast.success(`${ids.length} post-it rangés en ${frames.length} thèmes.`)
		);
	}

	function keydown(event: KeyboardEvent) {
		if (event.key === 'Escape') onclose();
		if (event.key === 'Enter' && !event.shiftKey && !event.isComposing && event.target instanceof HTMLTextAreaElement) generate(event);
	}

	function step(event: KeyboardEvent, i: number) {
		const move = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[event.key];
		if (!move) return;
		event.preventDefault();
		const next = (i + move + KINDS.length) % KINDS.length;
		kind = KINDS[next].value;
		chips[next]?.focus();
	}
</script>

<div
	bind:this={panel}
	role="dialog"
	tabindex="-1"
	class="toile-rise pointer-events-auto flex max-h-[60dvh] w-[min(28rem,calc(100vw-1.5rem))] flex-col gap-3 overflow-y-auto rounded-fc-lg bg-fc-component p-3 shadow-lg"
	aria-label={name}
	aria-busy={busy}
	onkeydown={keydown}
	data-board-ui
	data-testid="assistant"
>
	<header class="flex items-center gap-2">
		<Blob {name} size={32} animate={busy ? 'always' : undefined} expression={busy ? thinking : undefined} />
		<h2 class="flex-1 truncate text-fc-md font-semibold text-fc-fg">{name}</h2>
		<IconButton variant="ghost" aria-label="Fermer" onclick={onclose}><Icon icon={icons.close} size={18} /></IconButton>
	</header>

	{#if summary !== null}
		<div class="flex flex-col gap-3 rounded-fc-md bg-fc-surface p-3 text-fc-sm text-fc-fg" data-testid="ai-summary">
			{#each content as block, i (i)}
				{#if block.kind === 'heading'}
					<svelte:element this={`h${block.level + 1}`} class="{heading[block.level]} font-semibold">{block.text}</svelte:element>
				{:else if block.kind === 'list'}
					<ul class="flex list-disc flex-col gap-1 pl-5">
						{#each block.items as item, n (n)}<li>{item}</li>{/each}
					</ul>
				{:else}
					<p>{block.text}</p>
				{/if}
			{/each}
		</div>
	{/if}

	<p class="text-fc-xs text-fc-fg-muted" aria-live="polite">{busy ? `${name} réfléchit…` : ''}</p>

	<form class="flex flex-col gap-3" onsubmit={generate}>
		<Textarea
			bind:value={prompt}
			rows={2}
			maxlength={2000}
			aria-label="Que voulez-vous dessiner ?"
			placeholder="Demandez à {name} de dessiner, par exemple le lancement de la rentrée"
		/>
		<div class="flex flex-wrap gap-1" role="radiogroup" aria-label="Forme">
			{#each KINDS as option, i (option.value)}
				<Tooltip text={option.hint}>
					<button
						bind:this={chips[i]}
						type="button"
						role="radio"
						aria-checked={kind === option.value}
						tabindex={kind === option.value ? 0 : -1}
						class="min-h-11 cursor-pointer rounded-fc-pill px-3 text-fc-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring {kind ===
						option.value
							? 'bg-fc-accent text-fc-accent-fg'
							: 'bg-fc-surface text-fc-fg hover:bg-fc-field'}"
						onclick={() => (kind = option.value)}
						onkeydown={(event) => step(event, i)}>{option.label}</button
					>
				</Tooltip>
			{/each}
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<Button variant="secondary" size="sm" disabled={busy} onclick={summarize}>Résumer le tableau</Button>
			{#if stickies.length >= 2}
				<Button variant="secondary" size="sm" disabled={busy} onclick={cluster}>Ranger {stickies.length} post-it</Button>
			{/if}
			<Button type="submit" size="sm" class="ml-auto" disabled={busy || !prompt.trim()}>Dessiner</Button>
		</div>
	</form>
</div>

<PlanLimit error={limit} variant="dialog" onClose={() => (limit = null)} />
