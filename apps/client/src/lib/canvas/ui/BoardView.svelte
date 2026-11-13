<script lang="ts">
	import { goto } from '$app/navigation';
	import { Blob } from '@facile/blob';
	import { Alert, ConnectionHint, Icon, IconButton, Presence, Tooltip, icons } from '@facile/muse';
	import { assistantName } from '$lib/api/ai';
	import { boardApi, canEdit } from '$lib/board/api';
	import { presencePeers } from '$lib/board/presence';
	import type { BoardSession } from '$lib/board/session.svelte';
	import type { CommandUi } from '../commands';
	import { toolIcons } from '../icons';
	import type { Input } from '../input';
	import { pasteInto } from '../paste';
	import { thumbnailer } from '../thumbnail';
	import { zoomFit } from '../view';
	import Assistant from './Assistant.svelte';
	import BoardTitle from './BoardTitle.svelte';
	import Board from './Board.svelte';
	import EmbedBar from './EmbedBar.svelte';
	import Cheatsheet from './Cheatsheet.svelte';
	import CommentPanel from './CommentPanel.svelte';
	import CommentPins from './CommentPins.svelte';
	import Minimap from './Minimap.svelte';
	import SearchPalette from './SearchPalette.svelte';
	import ShareDialog from './ShareDialog.svelte';
	import StyleBar from './StyleBar.svelte';
	import Toolbar from './Toolbar.svelte';
	import ZoomControls from './ZoomControls.svelte';

	let {
		session,
		name,
		back,
		embed = false
	}: { session: BoardSession; name: string; back?: string; embed?: boolean } = $props();

	const editor = $derived(session.editor);
	let input = $state<Input>();
	let help = $state(false);
	let search = $state(false);
	let sharing = $state(false);
	let comments = $state(false);
	let minimap = $state(true);
	let thread = $state<string | null>(null);
	let assisting = $state(false);
	let assistant = $state('Bulle');
	let picker: HTMLInputElement;
	const editBoard = $derived('board' in session.target && canEdit(session.role) ? session.target.board : null);
	let fitted = false;
	const shots = $derived(editBoard ? thumbnailer(editor, (png) => boardApi.thumbnail(editBoard, png)) : null);

	$effect(() => {
		void editor.version;
		if (session.synced) shots?.schedule();
	});

	$effect(() => () => shots?.flush());

	const ui: CommandUi = {
		help: () => (help = true),
		search: () => (search = true),
		image: () => picker.click(),
		minimap: () => (minimap = !minimap)
	};

	$effect(() => {
		if (!session.synced || fitted || !editor.size.width) return;
		fitted = true;
		if (editor.scene.shapes.size) zoomFit(editor);
	});

	$effect(() => { if (session.draft) comments = true; });

	$effect(() => {
		if (editBoard) void assistantName(editBoard).then((found) => (assistant = found));
	});

	async function picked() {
		const files = picker.files;
		if (!files?.length) return;
		const data = new DataTransfer();
		for (const file of files) data.items.add(file);
		picker.value = '';
		await pasteInto(editor, data, null);
	}

	const chrome = 'pointer-events-auto flex items-center gap-1 rounded-fc-pill bg-fc-component p-1.5 shadow-lg';

	function closeAssistant() {
		assisting = false;
		document.querySelector<HTMLElement>('[data-testid="assistant-trigger"]')?.focus();
	}
</script>

<div
	class="relative h-dvh w-full overflow-hidden bg-fc-page"
	data-testid="board"
	data-role={session.role}
	data-connection={session.connection}
	data-editable={editor.editable}
>
	<Board {editor} {ui} bind:input />
	<div class="pointer-events-none absolute inset-0 overflow-hidden">
		<CommentPins {session} onopen={(id) => ((thread = id), (comments = true))} />
	</div>

	<div class="pointer-events-none absolute inset-0 flex flex-col justify-between gap-3 p-3">
		{#if embed && 'board' in session.target}
			<EmbedBar {session} href="/b/{session.target.board}" />
		{:else}
			<div class="flex items-start justify-between gap-2">
				<div class={chrome} data-board-ui>
					{#if back}
						<IconButton variant="ghost" aria-label="Retour aux tableaux" onclick={() => back && goto(back)}>
							<Icon icon={icons.chevronLeft} size={20} />
						</IconButton>
					{/if}
					<BoardTitle {name} board={editBoard} />
					<ConnectionHint state={session.connection} class="mr-1" />
				</div>
				<div class="flex items-center gap-2">
					<Presence
						peers={presencePeers(session.peers)}
						following={session.following}
						onfollow={(id) => session.follow(id === null ? null : Number(id))}
						class="pointer-events-auto"
						data-board-ui
					/>
					<div class={chrome} data-board-ui>
						<IconButton variant="ghost" aria-label="Rechercher" tooltip="Rechercher (⌘ K)" onclick={() => (search = true)}
							><Icon icon={icons.search} size={20} /></IconButton
						>
						<IconButton
							variant="ghost"
							aria-label="Commentaires"
							aria-pressed={comments}
							onclick={() => (comments = !comments)}
						>
							<Icon icon={toolIcons.comment} size={20} />
						</IconButton>
						<span class="hidden md:inline-flex">
							<IconButton variant="ghost" aria-label="Raccourcis clavier" tooltip="Raccourcis (?)" onclick={() => (help = true)}
								><Icon icon={toolIcons.keyboard} size={20} /></IconButton
							>
						</span>
						{#if session.role === 'owner' && 'board' in session.target}
							<IconButton variant="ghost" aria-label="Partager" onclick={() => (sharing = true)}
								><Icon icon={toolIcons.share} size={20} /></IconButton
							>
						{/if}
					</div>
				</div>
			</div>
		{/if}

		{#if session.stop}
			<Alert tone="danger" class="pointer-events-auto mx-auto max-w-md">{session.stopText}</Alert>
		{:else if !canEdit(session.role)}
			<p
				class="pointer-events-auto mx-auto rounded-fc-pill bg-fc-component px-4 py-2 text-fc-xs text-fc-fg-muted shadow-lg"
			>
				{session.role === 'commenter' ? 'Vous pouvez commenter ce tableau.' : 'Lecture seule.'}
			</p>
		{/if}

		<div class="flex min-h-0 flex-1 justify-end">
			{#if comments}
				<div class="absolute right-3 bottom-24 left-3 md:static md:h-full">
					<CommentPanel {session} bind:active={thread} onclose={() => (comments = false)} />
				</div>
			{/if}
		</div>

		<div class="flex flex-col items-center gap-2 md:flex-row md:items-end md:justify-between">
			<div class="hidden md:block md:w-48"></div>
			<div class="flex flex-col items-center gap-2">
				{#if assisting && editBoard}
					<Assistant board={editBoard} {editor} name={assistant} onclose={closeAssistant} />
				{/if}
				<StyleBar {editor} {ui} />
				<div class="flex max-w-[calc(100vw-1.5rem)] items-center gap-2">
					{#if editBoard}
						<Tooltip text={assistant} describe={false} class="contents">
							<button
								type="button"
								class="toile-rise toile-press pointer-events-auto flex size-14 shrink-0 cursor-pointer items-center justify-center rounded-fc-pill shadow-lg focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring {assisting
									? 'bg-fc-surface'
									: 'bg-fc-component hover:bg-fc-surface'}"
								aria-label={assistant}
								aria-haspopup="dialog"
								aria-expanded={assisting}
								data-testid="assistant-trigger"
								data-board-ui
								onclick={() => (assisting = !assisting)}
							>
								<Blob name={assistant} size={32} />
							</button>
						</Tooltip>
					{/if}
					<div class="min-w-0"><Toolbar {editor} onimage={() => picker.click()} /></div>
				</div>
			</div>
			<div class="flex flex-col items-end gap-2">
				{#if minimap && !embed}<span class="hidden lg:block"><Minimap {editor} /></span>{/if}
				<ZoomControls {editor} />
			</div>
		</div>
	</div>
</div>

<input
	bind:this={picker}
	type="file"
	accept="image/*"
	multiple
	class="hidden"
	onchange={picked}
	aria-hidden="true"
	tabindex="-1"
/>
<Cheatsheet bind:open={help} />
<SearchPalette {editor} bind:open={search} />
{#if 'board' in session.target}
	<ShareDialog board={session.target.board} bind:open={sharing} />
{/if}
