<script lang="ts">
	import { goto } from '$app/navigation';
	import { flip } from 'svelte/animate';
	import {
		Button,
		Card,
		Icon,
		ConfirmModal,
		EmptyState,
		Field,
		Input,
		Menu,
		Page,
		PageHeader,
		icons,
		toast
	} from '@facile/muse';
	import BoardPreview from '$lib/components/boards/BoardPreview.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import { boardApi, type BoardInfo } from '$lib/board/api';
	import { toolIcons } from '$lib/canvas/icons';
	import { boardIcon } from '$lib/brand';
	import { formatDate } from '$lib/format';
	import { session } from '$lib/session.svelte';

	let boards = $state<BoardInfo[]>([]);
	let loading = $state(true);
	let name = $state('');
	let busy = $state(false);
	let doomed = $state<BoardInfo | null>(null);
	let contextMenu = $state<{ x: number; y: number; board: BoardInfo } | null>(null);

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

<Page width="xl">
	<PageHeader title="Tableaux" description={session.space?.name ?? 'Espace personnel'} />

	<form class="toile-rise flex max-w-xl flex-wrap items-end gap-2" onsubmit={create}>
		<Field label="Nom du tableau" class="min-w-0 flex-1">
			<Input bind:value={name} placeholder="Atelier du lundi" maxlength={200} />
		</Field>
		<Button type="submit" size="lg" icon={icons.plus} disabled={busy}>Nouveau tableau</Button>
	</form>

	{#if boards.length === 0 && !loading}
		<EmptyState icon={boardIcon} title="Aucun tableau" description="Créez un premier tableau pour commencer." />
	{:else}
		<ul class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4" aria-label="Tableaux">
			{#each boards as board, i (board.id)}
				<li
					class="toile-pop group/board relative"
					style:--i={Math.min(i, 12)}
					animate:flip={{ duration: 300 }}
					oncontextmenu={(e) => {
						e.preventDefault();
						contextMenu = { x: e.clientX, y: e.clientY, board };
					}}
				>
					<Card
						href="/b/{board.id}"
						class="flex h-full flex-col gap-3 p-3 transition-[background-color,translate] duration-300 hover:-translate-y-1 motion-reduce:transition-none"
					>
						<BoardPreview seed={board.id} thumbnail={board.thumbnail_url} />
						<span class="flex min-w-0 flex-col gap-1 px-1 pb-1">
							<span class="truncate text-fc-sm font-medium text-fc-fg">{board.name}</span>
							<span class="text-fc-xs text-fc-fg-muted">Modifié le {formatDate(board.updated_at)}</span>
						</span>
					</Card>
					<Menu
						label="Actions pour {board.name}"
						items={actions(board)}
						align="end"
						class="absolute right-5 top-5 rounded-fc-md shadow-sm transition-opacity focus-within:opacity-100 group-hover/board:opacity-100 motion-reduce:transition-none [@media(hover:hover)_and_(pointer:fine)]:opacity-0"
					>
						{#snippet trigger()}<Icon icon={toolIcons.dots} size={20} />{/snippet}
					</Menu>
				</li>
			{/each}
		</ul>
	{/if}
</Page>

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

{#if contextMenu}
	<ContextMenu
		x={contextMenu.x}
		y={contextMenu.y}
		items={actions(contextMenu.board).map((a) => ({
			label: a.label,
			icon: a.icon,
			tone: a.tone,
			onselect: a.onselect ?? (() => {})
		}))}
		onclose={() => (contextMenu = null)}
	/>
{/if}
