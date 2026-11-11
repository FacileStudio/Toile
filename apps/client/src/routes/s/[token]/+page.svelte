<script lang="ts">
	import { page } from '$app/state';
	import { EmptyState, Spinner } from '@facile/muse';
	import { boardApi, roleOfShare, type BoardInfo } from '$lib/board/api';
	import { guestName } from '$lib/board/guest';
	import { colorFor } from '$lib/board/presence';
	import { BoardSession } from '$lib/board/session.svelte';
	import { boardIcon } from '$lib/brand';
	import { session as account } from '$lib/session.svelte';
	import BoardView from '$lib/canvas/ui/BoardView.svelte';
	import GuestName from '$lib/canvas/ui/GuestName.svelte';

	let board = $state<BoardInfo | null>(null);
	let role = $state<BoardInfo['role']>('viewer');
	let missing = $state(false);
	let name = $state(guestName() || account.user?.name || '');
	let named = $state(!!guestName());
	let open = $state<BoardSession | null>(null);

	const token = $derived(page.params.token ?? '');

	$effect(() => {
		let cancelled = false;
		boardApi
			.resolve(token)
			.then((answer) => {
				if (cancelled) return;
				board = answer.board;
				role = roleOfShare(answer.role);
			})
			.catch(() => (missing = !cancelled));
		return () => {
			cancelled = true;
		};
	});

	$effect(() => {
		if (!board || !named) return;
		const me = { name, color: colorFor(name), guest: true };
		const live = (open = new BoardSession({ token, name }, role, me, board.id));
		return () => {
			live.destroy();
			open = null;
		};
	});
</script>

<svelte:head><title>{board?.name ?? 'Tableau partagé'} · Toile</title></svelte:head>

{#if missing}
	<div class="flex h-dvh items-center justify-center bg-fc-page p-6">
		<EmptyState icon={boardIcon} title="Lien invalide" description="Ce lien a été révoqué ou n’existe pas." />
	</div>
{:else if !board}
	<div class="flex h-dvh items-center justify-center bg-fc-page"><Spinner /></div>
{:else if !named}
	<GuestName bind:name board={board.name} onconfirm={() => (named = true)} />
{:else if open}
	<BoardView session={open} name={board.name} />
{/if}
