<script lang="ts">
	import { Icon, Tooltip } from '@facile/muse';
	import type { Anchor, Comment } from '$lib/board/api';
	import type { BoardSession } from '$lib/board/session.svelte';
	import { toScreen } from '../camera';
	import { toolIcons } from '../icons';

	let { session, onopen }: { session: BoardSession; onopen: (id: string) => void } = $props();

	const editor = $derived(session.editor);

	function place(anchor: Anchor) {
		void editor.version;
		const base = anchor.shape ? editor.scene.get(anchor.shape) : null;
		const world = base ? { x: base.x + anchor.x, y: base.y + anchor.y } : anchor;
		return toScreen(editor.camera, world);
	}

	const roots = $derived(session.comments.filter((c: Comment) => !c.parent && !c.resolved_at));
	const count = (id: string) => session.comments.filter((c) => c.parent === id).length + 1;
</script>

{#each roots as comment (comment.id)}
	{@const at = place(comment.anchor)}
	<Tooltip text="Commentaire de {comment.author}" describe={false} class="contents">
		<button
			type="button"
			class="pointer-events-auto absolute top-0 left-0 flex h-9 min-w-9 items-center gap-1 rounded-fc-pill rounded-bl-fc-xs bg-fc-accent px-2 text-fc-xs font-medium text-fc-accent-fg shadow-lg focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
			style:transform="translate({at.x}px, {at.y}px) translateY(-100%)"
			aria-label="Commentaire de {comment.author}"
			onclick={() => onopen(comment.id)}
			data-board-ui
		>
			<Icon icon={toolIcons.comment} size={16} />{count(comment.id)}
		</button>
	</Tooltip>
{/each}
{#if session.draft}
	{@const at = place(session.draft)}
	<span
		class="pointer-events-none absolute top-0 left-0 size-9 rounded-fc-pill rounded-bl-fc-xs bg-fc-info shadow-lg"
		style:transform="translate({at.x}px, {at.y}px) translateY(-100%)"
	></span>
{/if}
