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
		Modal,
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
	import { t } from '$lib/i18n/index.svelte';

	let boards = $state<BoardInfo[]>([]);
	let loading = $state(true);
	let name = $state('');
	let busy = $state(false);
	let doomed = $state<BoardInfo | null>(null);
	let renaming = $state<BoardInfo | null>(null);
	let renameTitle = $state('');
	let contextMenu = $state<{ x: number; y: number; board: BoardInfo } | null>(null);
	let gridEl = $state<HTMLElement | null>(null);
	let cols = $state(1);

	const failed = (error: unknown, fallback: string) => toast.danger(error instanceof Error ? error.message : fallback);

	async function load(spaceId: string | null) {
		loading = true;
		try {
			boards = await boardApi.list(spaceId);
		} catch (error) {
			failed(error, t('boards.loadFailed'));
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		void load(session.spaceId);
	});

	$effect(() => {
		if (!gridEl) return;
		const update = () => {
			const style = window.getComputedStyle(gridEl!).getPropertyValue('grid-template-columns');
			cols = style ? style.split(' ').filter(Boolean).length : 1;
		};
		update();
		const ro = new ResizeObserver(update);
		ro.observe(gridEl);
		return () => ro.disconnect();
	});

	const showAddCard = $derived(cols > 1 && (boards.length % cols !== 0));

	async function create(event?: SubmitEvent, customTitle?: string) {
		event?.preventDefault();
		busy = true;
		try {
			const title = customTitle ?? (name.trim() || t('common.untitled'));
			const board = await boardApi.create(title, session.spaceId);
			boards = [board, ...boards];
			name = '';
			if (customTitle) await goto(`/b/${board.id}`);
		} catch (error) {
			failed(error, t('boards.createFailed'));
		} finally {
			busy = false;
		}
	}

	async function saveRename(event: SubmitEvent) {
		event.preventDefault();
		const board = renaming;
		const trimmed = renameTitle.trim();
		if (!board || !trimmed) return;
		try {
			const updated = await boardApi.rename(board.id, trimmed);
			boards = boards.map((b) => (b.id === board.id ? { ...b, name: updated.name } : b));
			renaming = null;
			toast.success(t('boards.renamed'));
		} catch (error) {
			failed(error, t('boards.renameFailed'));
		}
	}

	async function duplicate(board: BoardInfo) {
		try {
			boards = [await boardApi.duplicate(board.id), ...boards];
		} catch (error) {
			failed(error, t('boards.copyFailed'));
		}
	}

	async function remove() {
		const board = doomed;
		if (!board) return;
		await boardApi.remove(board.id);
		boards = boards.filter((b) => b.id !== board.id);
		doomed = null;
		toast.success(t('boards.trashed'));
	}

	const actions = (board: BoardInfo) => [
		{ label: t('boards.open'), icon: icons.external, onselect: () => goto(`/b/${board.id}`) },
		{
			label: t('boards.rename'),
			icon: icons.edit,
			onselect: () => {
				renaming = board;
				renameTitle = board.name;
			}
		},
		{ label: t('boards.duplicate'), icon: icons.copy, onselect: () => duplicate(board) },
		...(board.role === 'owner'
			? [{ label: t('common.delete'), icon: icons.remove, tone: 'danger' as const, onselect: () => (doomed = board) }]
			: [])
	];
</script>

<svelte:head><title>{t('boards.title')} · Toile</title></svelte:head>

<Page width="xl" class="pb-24 md:pb-32">
	<PageHeader title={t('boards.title')} description={session.space?.name ?? t('boards.personal')} />

	<form class="toile-rise flex max-w-xl flex-wrap items-end gap-2" onsubmit={create}>
		<Field label={t('boards.name')} class="min-w-0 flex-1">
			<Input bind:value={name} placeholder={t('boards.namePlaceholder')} maxlength={200} />
		</Field>
		<Button type="submit" size="lg" icon={icons.plus} disabled={busy}>{t('boards.new')}</Button>
	</form>

	{#if boards.length === 0 && !loading}
		<EmptyState icon={boardIcon} title={t('boards.empty')} description={t('boards.emptyBody')} />
	{:else}
		<ul
			bind:this={gridEl}
			class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
			aria-label={t('boards.title')}
		>
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
						class="flex h-full flex-col gap-3 p-3 transition-[background-color,translate] duration-300 hover:-translate-y-1 motion-reduce:transition-none cursor-pointer"
					>
						<BoardPreview seed={board.id} thumbnail={board.thumbnail_url} />
						<span class="flex min-w-0 flex-col gap-1 px-1 pb-1">
							<span class="truncate text-fc-sm font-medium text-fc-fg">{board.name}</span>
							<span class="text-fc-xs text-fc-fg-muted">{t('boards.updated', { date: formatDate(board.updated_at) })}</span>
						</span>
					</Card>
					<Menu
						label={t('boards.actions', { name: board.name })}
						items={actions(board)}
						align="end"
						class="absolute right-5 top-5 rounded-fc-md shadow-sm transition-opacity focus-within:opacity-100 group-hover/board:opacity-100 motion-reduce:transition-none [@media(hover:hover)_and_(pointer:fine)]:opacity-0"
					>
						{#snippet trigger()}<Icon icon={toolIcons.dots} size={20} />{/snippet}
					</Menu>
				</li>
			{/each}
			{#if showAddCard}
				<li class="h-full">
					<button
						type="button"
						onclick={() => void create(undefined, t('common.untitled'))}
						disabled={busy}
						class="toile-pop group flex h-full min-h-[220px] w-full flex-col items-center justify-center gap-3 rounded-fc-lg bg-fc-surface/50 p-4 text-center transition-[background-color,translate] duration-300 hover:-translate-y-1 hover:bg-fc-surface focus-visible:ring-2 focus-visible:ring-fc-accent motion-reduce:transition-none cursor-pointer disabled:cursor-not-allowed"
						aria-label={t('boards.new')}
					>
						<span
							class="flex size-10 items-center justify-center rounded-full bg-fc-component text-fc-fg-muted transition-colors group-hover:bg-fc-page group-hover:text-fc-fg"
						>
							<Icon icon={icons.plus} size={20} />
						</span>
						<span class="flex flex-col gap-0.5">
							<span class="text-fc-sm font-medium text-fc-fg">{t('boards.new')}</span>
							<span class="text-fc-xs text-fc-fg-muted">{t('boards.create')}</span>
						</span>
					</button>
				</li>
			{/if}
		</ul>
	{/if}
</Page>

<ConfirmModal
	open={!!doomed}
	title={t('boards.deleteTitle', { name: doomed?.name ?? '' })}
	description={t('boards.deleteBody')}
	confirmLabel={t('common.delete')}
	cancelLabel={t('common.cancel')}
	tone="danger"
	onConfirm={remove}
	onCancel={() => (doomed = null)}
/>

{#if renaming}
	<Modal open={!!renaming} title={t('boards.renameTitle')} onClose={() => (renaming = null)}>
		<form id="rename-board" class="flex flex-col gap-4" onsubmit={saveRename}>
			<Field label={t('boards.name')} helper={t('boards.nameHelper')}>
				{#snippet children({ id, describedBy })}
					<Input {id} aria-describedby={describedBy} bind:value={renameTitle} maxlength={200} autocomplete="off" />
				{/snippet}
			</Field>
		</form>
		{#snippet footer()}
			<Button variant="ghost" onclick={() => (renaming = null)}>{t('common.cancel')}</Button>
			<Button type="submit" form="rename-board" disabled={!renameTitle.trim()}>{t('common.save')}</Button>
		{/snippet}
	</Modal>
{/if}

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
