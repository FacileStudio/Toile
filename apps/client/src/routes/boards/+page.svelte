<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button, Card, Icon, ConfirmModal, EmptyState, Field, Input, Menu, icons, toast } from '@facile/muse';
	import { boardApi, type BoardInfo } from '$lib/board/api';
	import { boardIcon } from '$lib/brand';
	import { formatDate } from '$lib/format';
	import { session } from '$lib/session.svelte';

	let boards = $state<BoardInfo[]>([]);
	let loading = $state(true);
	let name = $state('');
	let busy = $state(false);
	let doomed = $state<BoardInfo | null>(null);

	const failed = (error: unknown, fallback: string) => toast.danger(error instanceof Error ? error.message : fallback);

	async function load(spaceId: string | null) {
		loading = true;
		try {
			boards = await boardApi.list(spaceId);
		} catch (error) {
			failed(error, 'Chargement impossible');
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
			const board = await boardApi.create(name.trim() || 'Sans titre', session.spaceId);
			boards = [board, ...boards];
			name = '';
		} catch (error) {
			failed(error, 'Création impossible');
		} finally {
			busy = false;
		}
	}

	async function duplicate(board: BoardInfo) {
		try {
			boards = [await boardApi.duplicate(board.id), ...boards];
		} catch (error) {
			failed(error, 'Copie impossible');
		}
	}

	async function remove() {
		const board = doomed;
		if (!board) return;
		await boardApi.remove(board.id);
		boards = boards.filter((b) => b.id !== board.id);
		doomed = null;
		toast.success('Tableau placé dans la corbeille.');
	}

	const actions = (board: BoardInfo) => [
		{ label: 'Ouvrir', icon: icons.external, onselect: () => goto(`/b/${board.id}`) },
		{ label: 'Dupliquer', icon: icons.copy, onselect: () => duplicate(board) },
		...(board.role === 'owner'
			? [{ label: 'Supprimer', icon: icons.remove, tone: 'danger' as const, onselect: () => (doomed = board) }]
			: [])
	];
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
				<li class="flex items-center gap-2">
					<Card href="/b/{board.id}" class="flex min-w-0 flex-1 items-center gap-3">
						<span class="min-w-0 flex-1 truncate text-fc-sm font-medium text-fc-fg">{board.name}</span>
						<span class="shrink-0 text-fc-xs text-fc-fg-muted">{formatDate(board.updated_at)}</span>
					</Card>
					<Menu label="Actions pour {board.name}" items={actions(board)} align="end">
						{#snippet trigger()}<Icon icon={icons.menu} size={18} />{/snippet}
					</Menu>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<ConfirmModal
	open={!!doomed}
	title="Supprimer « {doomed?.name ?? ''} » ?"
	description="Le tableau part dans la corbeille ; ses liens de partage cessent de fonctionner."
	confirmLabel="Supprimer"
	cancelLabel="Annuler"
	tone="danger"
	onConfirm={remove}
	onCancel={() => (doomed = null)}
/>
