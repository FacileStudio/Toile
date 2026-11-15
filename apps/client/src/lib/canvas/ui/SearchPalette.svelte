<script lang="ts">
	import { locale, t, type AppKey } from '$lib/i18n/index.svelte';
	import { Input, Modal } from '@facile/muse';
	import type { Shape } from '$lib/board/schema';
	import type { Editor } from '../editor.svelte';
	import { rotatedBox } from '../geometry';
	import { showBox } from '../view';

	let { editor, open = $bindable(false) }: { editor: Editor; open?: boolean } = $props();

	let query = $state('');
	let active = $state(0);

	const label = (shape: Shape) => shape.text || shape.name || shape.title || shape.url || '';
	const KIND: Partial<Record<Shape['type'], AppKey>> = {
		sticky: 'search.sticky',
		text: 'search.text',
		frame: 'search.frame',
		card: 'search.card'
	};

	const results = $derived.by(() => {
		void editor.version;
		const needle = query.trim().toLocaleLowerCase(locale());
		if (!needle) return editor.scene.order.filter((s) => s.type === 'frame').slice(0, 20);
		return editor.scene.order.filter((s) => label(s).toLocaleLowerCase(locale()).includes(needle)).slice(0, 20);
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

<Modal bind:open title={t('search.title')} showClose>
	<div class="flex flex-col gap-3">
		<Input
			bind:value={query}
			placeholder={t('search.placeholder')}
			aria-label={t('search.label')}
			onkeydown={keydown}
			oninput={() => (active = 0)}
		/>
		{#if results.length}
			<ul class="flex max-h-80 flex-col gap-1 overflow-y-auto" role="listbox" aria-label={t('search.results')}>
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
							<span class="shrink-0 text-fc-xs text-fc-fg-muted">{t(KIND[shape.type] ?? 'search.shape')}</span>
							<span class="min-w-0 flex-1 truncate text-fc-fg">{label(shape) || 'Sans titre'}</span>
						</button>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="py-4 text-center text-fc-sm text-fc-fg-muted">{t('search.none')}</p>
		{/if}
	</div>
</Modal>
