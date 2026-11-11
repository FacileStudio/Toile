<script lang="ts">
	import { Button, IconButton, Icon, Textarea, icons, toast } from '@facile/muse';
	import { canComment, type Comment } from '$lib/board/api';
	import type { BoardSession } from '$lib/board/session.svelte';
	import { formatDate } from '$lib/format';

	let {
		session,
		active = $bindable(null),
		onclose
	}: { session: BoardSession; active?: string | null; onclose: () => void } = $props();

	let body = $state('');
	let reply = $state('');
	let busy = $state(false);
	let showResolved = $state(false);

	const may = $derived(canComment(session.role));
	const roots = $derived(session.comments.filter((c) => !c.parent && (showResolved || !c.resolved_at)));
	const repliesOf = (id: string) => session.comments.filter((c) => c.parent === id);
	const mine = (c: Comment) => c.mine;

	async function attempt(action: () => Promise<void>) {
		busy = true;
		try {
			await action();
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Action impossible');
		} finally {
			busy = false;
		}
	}

	function post(event: SubmitEvent) {
		event.preventDefault();
		const anchor = session.draft;
		if (!anchor || !body.trim()) return;
		void attempt(async () => {
			await session.addComment(body.trim(), anchor);
			body = '';
		});
	}

	function answer(event: SubmitEvent, root: Comment) {
		event.preventDefault();
		if (!reply.trim()) return;
		void attempt(async () => {
			await session.addComment(reply.trim(), root.anchor, root.id);
			reply = '';
		});
	}
</script>

{#snippet entry(c: Comment)}
	<div class="flex flex-col gap-1">
		<div class="flex items-center gap-2 text-fc-xs text-fc-fg-muted">
			<span class="font-medium text-fc-fg">{c.author}{c.guest ? ' (invité)' : ''}</span>
			<span>{formatDate(c.created_at)}</span>
			{#if may && (mine(c) || session.role === 'owner')}
				<button
					type="button"
					class="ml-auto rounded-fc-xs px-1 hover:text-fc-danger"
					onclick={() => attempt(() => session.removeComment(c.id))}
				>
					Supprimer
				</button>
			{/if}
		</div>
		<p class="text-fc-sm whitespace-pre-wrap text-fc-fg">{c.body}</p>
	</div>
{/snippet}

<aside
	class="pointer-events-auto flex max-h-[70dvh] w-full flex-col gap-4 overflow-y-auto rounded-fc-lg bg-fc-component p-4 shadow-lg md:max-h-full md:w-80"
	aria-label="Commentaires"
	data-board-ui
>
	<header class="flex items-center justify-between gap-2">
		<h2 class="text-fc-md font-semibold text-fc-fg">Commentaires</h2>
		<IconButton variant="ghost" aria-label="Fermer les commentaires" onclick={onclose}
			><Icon icon={icons.close} size={18} /></IconButton
		>
	</header>
	{#if session.draft && may}
		<form class="flex flex-col gap-2" onsubmit={post}>
			<Textarea
				bind:value={body}
				rows={3}
				maxlength={4000}
				placeholder="Écrire un commentaire…"
				aria-label="Nouveau commentaire"
			/>
			<div class="flex justify-end gap-2">
				<Button variant="ghost" onclick={() => (session.draft = null)}>Annuler</Button>
				<Button type="submit" disabled={busy || !body.trim()}>Publier</Button>
			</div>
		</form>
	{:else if may && !roots.length}
		<p class="text-fc-sm text-fc-fg-muted">Choisissez l’outil Commentaire (C), puis touchez le tableau.</p>
	{/if}
	<ul class="flex flex-col gap-3">
		{#each roots as root (root.id)}
			<li class="flex flex-col gap-3 rounded-fc-md p-3 {active === root.id ? 'bg-fc-surface' : ''}">
				<button type="button" class="text-left" onclick={() => (active = root.id)}>{@render entry(root)}</button>
				{#each repliesOf(root.id) as child (child.id)}
					<div class="pl-3">{@render entry(child)}</div>
				{/each}
				{#if may && active === root.id}
					<form class="flex flex-col gap-2" onsubmit={(e) => answer(e, root)}>
						<Textarea bind:value={reply} rows={2} maxlength={4000} placeholder="Répondre…" aria-label="Réponse" />
						<div class="flex justify-end gap-2">
							<Button variant="ghost" onclick={() => attempt(() => session.resolveComment(root.id, !root.resolved_at))}>
								{root.resolved_at ? 'Rouvrir' : 'Résoudre'}
							</Button>
							<Button type="submit" disabled={busy || !reply.trim()}>Répondre</Button>
						</div>
					</form>
				{/if}
			</li>
		{/each}
	</ul>
	<button
		type="button"
		class="self-start text-fc-xs text-fc-fg-muted hover:text-fc-fg"
		onclick={() => (showResolved = !showResolved)}
	>
		{showResolved ? 'Masquer les fils résolus' : 'Afficher les fils résolus'}
	</button>
</aside>
