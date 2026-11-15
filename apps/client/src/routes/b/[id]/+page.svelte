<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import { EmptyState, Spinner } from '@facile/muse';
	import { boardApi, type BoardInfo } from '$lib/board/api';
	import { colorFor } from '$lib/board/presence';
	import { BoardSession } from '$lib/board/session.svelte';
	import { boardIcon } from '$lib/brand';
	import { t } from '$lib/i18n/index.svelte';
	import { session as account } from '$lib/session.svelte';
	import BoardView from '$lib/canvas/ui/BoardView.svelte';

	let board = $state<BoardInfo | null>(null);
	let missing = $state(false);
	let open = $state<BoardSession | null>(null);
	const embed = $derived(page.url.searchParams.get('embed') === '1');

	$effect(() => {
		const id = page.params.id ?? '';
		const user = untrack(() => account.user);
		let live: BoardSession | null = null;
		let cancelled = false;
		boardApi
			.get(id)
			.then((info) => {
				if (cancelled) return;
				board = info;
				const name = user?.name || user?.email || t('board.member');
				const me = { name, color: colorFor(user?.email ?? name), picture: user?.avatar_url || undefined, guest: false };
				live = open = new BoardSession({ board: id }, info.role, me, id);
			})
			.catch(() => (missing = !cancelled));
		return () => {
			cancelled = true;
			live?.destroy();
			open = null;
		};
	});
</script>

<svelte:head><title>{board?.name ?? t('board.label')} · Toile</title></svelte:head>

{#if open && board}
	<BoardView session={open} name={board.name} back={embed ? undefined : '/boards'} {embed} />
{:else if missing}
	<div class="flex h-dvh items-center justify-center bg-fc-page p-6">
		<EmptyState
			icon={boardIcon}
			title={t('boards.missing')}
			description={t('boards.missingBody')}
		/>
	</div>
{:else}
	<div class="flex h-dvh items-center justify-center bg-fc-page"><Spinner /></div>
{/if}
