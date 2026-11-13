<script lang="ts">
	import { Icon, icons } from '@facile/muse';
	import type { BoardSession } from '$lib/board/session.svelte';
	import AvatarStack from './AvatarStack.svelte';

	let { session, href }: { session: BoardSession; href: string } = $props();
</script>

<div class="flex items-start justify-end gap-2">
	<AvatarStack {session} />
	<a
		{href}
		target="_blank"
		rel="noopener"
		class="pointer-events-auto flex h-11 items-center gap-2 rounded-fc-pill bg-fc-component px-4 text-fc-sm font-medium text-fc-fg shadow-lg hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
		data-board-ui
	>
		Ouvrir dans Toile
		<Icon icon={icons.external} size={16} />
	</a>
</div>
