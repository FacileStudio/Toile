<script lang="ts">
	import { Button, Card, EmptyState, Field, Input, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { boardIcon } from '$lib/brand';
	import { formatDate } from '$lib/format';
	import { session } from '$lib/session.svelte';
	import type { Board } from '$lib/types';

	let boards = $state<Board[]>([]);
	let loading = $state(true);
	let name = $state('');
	let busy = $state(false);

	async function load(spaceId: string | null) {
		loading = true;
		try {
			boards = await backend.boards(spaceId);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Chargement impossible');
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		void load(session.spaceId);
	});

	async function create(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		try {
			const board = await backend.createBoard({ name, space_id: session.spaceId ?? undefined });
			boards = [board, ...boards];
			name = '';
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Création impossible');
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head><title>Tableaux · Toile</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	<header class="flex flex-col gap-1">
		<h1 class="text-fc-2xl font-semibold text-fc-fg">Tableaux</h1>
		<p class="text-fc-sm text-fc-fg-muted">{session.space?.name ?? 'Espace personnel'}</p>
	</header>

	<form class="flex flex-wrap items-end gap-2" onsubmit={create}>
		<Field label="Nom du tableau" class="min-w-0 flex-1">
			<Input bind:value={name} placeholder="Atelier du lundi" maxlength={200} />
		</Field>
		<Button type="submit" icon={icons.plus} disabled={busy}>Nouveau tableau</Button>
	</form>

	{#if boards.length === 0 && !loading}
		<EmptyState icon={boardIcon} title="Aucun tableau" description="Créez un premier tableau pour commencer." />
	{:else}
		<ul class="flex flex-col gap-2" aria-label="Tableaux">
			{#each boards as board (board.id)}
				<li>
					<Card class="flex items-center gap-3">
						<span class="min-w-0 flex-1 truncate text-fc-sm font-medium text-fc-fg">{board.name}</span>
						<span class="shrink-0 text-fc-xs text-fc-fg-muted">{formatDate(board.updated_at)}</span>
					</Card>
				</li>
			{/each}
		</ul>
	{/if}
</div>
