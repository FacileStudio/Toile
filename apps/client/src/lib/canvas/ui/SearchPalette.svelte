<script lang="ts">
	import { Input, Modal } from '@facile/muse';
	import type { Shape } from '$lib/board/schema';
	import type { Editor } from '../editor.svelte';
	import { rotatedBox } from '../geometry';
	import { showBox } from '../view';

	let { editor, open = $bindable(false) }: { editor: Editor; open?: boolean } = $props();

	let query = $state('');
	let active = $state(0);

	const label = (shape: Shape) => shape.text || shape.name || shape.title || shape.url || '';
	const KIND: Partial<Record<Shape['type'], string>> = { sticky: 'Note', text: 'Texte', frame: 'Cadre', card: 'Lien' };

	const results = $derived.by(() => {
		void editor.version;
		const needle = query.trim().toLocaleLowerCase('fr');
		if (!needle) return editor.scene.order.filter((s) => s.type === 'frame').slice(0, 20);
		return editor.scene.order.filter((s) => label(s).toLocaleLowerCase('fr').includes(needle)).slice(0, 20);
	});

	function go(shape: Shape) {
		open = false;
		query = '';
		showBox(editor, rotatedBox(shape), 1.5);
		editor.select([shape.id]);
	}

	function keydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') active = Math.min(results.length - 1, active + 1);
		else if (event.key === 'ArrowUp') active = Math.max(0, active - 1);
		else if (event.key === 'Enter' && results[active]) go(results[active]);
		else return;
		event.preventDefault();
	}
</script>

<Modal bind:open title="Rechercher dans le tableau" showClose>
	<div class="flex flex-col gap-3">
		<Input
			bind:value={query}
			placeholder="Texte, cadre, lien…"
			aria-label="Rechercher"
			onkeydown={keydown}
			oninput={() => (active = 0)}
		/>
		{#if results.length}
			<ul class="flex max-h-80 flex-col gap-1 overflow-y-auto" role="listbox" aria-label="Résultats">
				{#each results as shape, i (shape.id)}
					<li role="option" aria-selected={i === active}>
						<button
							type="button"
							class="flex w-full items-center gap-3 rounded-fc-md px-3 py-2.5 text-left text-fc-sm focus-visible:outline-2 focus-visible:outline-fc-ring {i ===
							active
								? 'bg-fc-surface'
								: 'hover:bg-fc-surface'}"
							onclick={() => go(shape)}
						>
							<span class="shrink-0 text-fc-xs text-fc-fg-muted">{KIND[shape.type] ?? 'Forme'}</span>
							<span class="min-w-0 flex-1 truncate text-fc-fg">{label(shape) || 'Sans titre'}</span>
						</button>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="py-4 text-center text-fc-sm text-fc-fg-muted">Aucun résultat.</p>
		{/if}
	</div>
</Modal>
