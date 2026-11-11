<script lang="ts">
	import { Avatar, Tooltip } from '@facile/muse';
	import type { BoardSession } from '$lib/board/session.svelte';

	let { session }: { session: BoardSession } = $props();

	const MAX = 4;
	const shown = $derived(session.peers.slice(0, MAX));
	const rest = $derived(session.peers.length - shown.length);
	const leader = $derived(session.peers.find((peer) => peer.id === session.following));
</script>

<div class="pointer-events-auto flex items-center gap-2" data-board-ui>
	{#if leader}
		<button
			type="button"
			class="h-11 rounded-fc-pill bg-fc-component px-4 text-fc-sm text-fc-fg shadow-lg focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
			onclick={() => session.follow(null)}
		>
			Vous suivez {leader.user.name} · Arrêter
		</button>
	{/if}
	<ul class="flex items-center -space-x-2" aria-label="Personnes sur le tableau">
		{#each shown as peer (peer.id)}
			<li>
				<Tooltip
					text={session.following === peer.id ? `Arrêter de suivre ${peer.user.name}` : `Suivre ${peer.user.name}`}
				>
					<button
						type="button"
						class="flex size-11 items-center justify-center rounded-fc-pill focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
						aria-label={session.following === peer.id
							? `Arrêter de suivre ${peer.user.name}`
							: `Suivre ${peer.user.name}`}
						aria-pressed={session.following === peer.id}
						onclick={() => session.follow(session.following === peer.id ? null : peer.id)}
					>
						<span class="rounded-fc-pill p-0.5" style:background-color={peer.user.color}>
							<Avatar size="sm" name={peer.user.name} src={peer.user.picture} />
						</span>
					</button>
				</Tooltip>
			</li>
		{/each}
		{#if rest > 0}
			<li class="flex size-9 items-center justify-center rounded-fc-pill bg-fc-surface text-fc-xs text-fc-fg">
				+{rest}
			</li>
		{/if}
	</ul>
</div>
