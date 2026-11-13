<script lang="ts">
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { Alert, Icon, IconButton, Spinner, Tooltip, icons, toast, type PlanLimitError } from '@facile/muse';
	import { aiError, aiPlanLimit, summarizeBoard } from '$lib/api/ai';
	import { boardApi, canEdit } from '$lib/board/api';
	import type { BoardSession } from '$lib/board/session.svelte';
	import type { CommandUi } from '../commands';
	import { toolIcons } from '../icons';
	import type { Input } from '../input';
	import { pasteInto } from '../paste';
	import { zoomFit } from '../view';
	import AiMenu from './AiMenu.svelte';
	import AiPanel from './AiPanel.svelte';
	import AvatarStack from './AvatarStack.svelte';
	import Board from './Board.svelte';
	import Cheatsheet from './Cheatsheet.svelte';
	import CommentPanel from './CommentPanel.svelte';
	import CommentPins from './CommentPins.svelte';
	import Minimap from './Minimap.svelte';
	import SearchPalette from './SearchPalette.svelte';
	import ShareDialog from './ShareDialog.svelte';
	import StyleBar from './StyleBar.svelte';
	import Toolbar from './Toolbar.svelte';
	import ZoomControls from './ZoomControls.svelte';

	let { session, name, back }: { session: BoardSession; name: string; back?: string } = $props();

	const editor = $derived(session.editor);
	let input = $state<Input>();
	let help = $state(false);
	let search = $state(false);
	let sharing = $state(false);
	let comments = $state(false);
	let minimap = $state(true);
	let thread = $state<string | null>(null);
	let summary = $state<string | null>(null);
	let thinking = $state(false);
	let limit = $state<PlanLimitError | null>(null);
	let picker: HTMLInputElement;
	let editingTitle = $state(false);
	let currentTitle = $state('');
	let titleInput = $state<HTMLInputElement>();
	const aiBoard = $derived('board' in session.target && canEdit(session.role) ? session.target.board : null);
	let fitted = false;

	$effect(() => { currentTitle = name; });

	async function saveTitle() {
		if (!editingTitle) return;
		editingTitle = false;
		const trimmed = currentTitle.trim();
		if (!trimmed || trimmed === name || !('board' in session.target)) return;
		try {
			await boardApi.rename(session.target.board, trimmed);
			name = trimmed;
		} catch {
			currentTitle = name;
			toast.danger('Renommage impossible');
		}
	}

	function startEditTitle() {
		if (!canEdit(session.role)) return;
		editingTitle = true;
		void tick().then(() => titleInput?.select());
	}

	const ui: CommandUi = {
		help: () => (help = true),
		search: () => (search = true),
		image: () => picker.click(),
		minimap: () => (minimap = !minimap)
	};
	const STATUS = { connecting: 'Connexion…', live: 'En ligne', offline: 'Hors ligne', stopped: 'Arrêté' } as const;

	$effect(() => {
		if (!session.synced || fitted || !editor.size.width) return;
		fitted = true;
		if (editor.scene.shapes.size) zoomFit(editor);
	});

	$effect(() => { if (session.draft) comments = true; });

	async function summarize() {
		if (!aiBoard) return;
		comments = false;
		summary = '';
		thinking = true;
		try {
			summary = await summarizeBoard(aiBoard);
		} catch (error) {
			summary = null;
			limit = aiPlanLimit(error);
			if (!limit) toast.danger(aiError(error));
		} finally {
			thinking = false;
		}
	}

	async function picked() {
		const files = picker.files;
		if (!files?.length) return;
		const data = new DataTransfer();
		for (const file of files) data.items.add(file);
		picker.value = '';
		await pasteInto(editor, data, null);
	}

	const chrome = 'pointer-events-auto flex items-center gap-1 rounded-fc-pill bg-fc-component p-1.5 shadow-lg';
</script>

<div
	class="relative h-dvh w-full overflow-hidden bg-fc-page"
	data-testid="board"
	data-role={session.role}
	data-editable={editor.editable}
>
	<Board {editor} {ui} bind:input />
	<div class="pointer-events-none absolute inset-0 overflow-hidden">
		<CommentPins {session} onopen={(id) => ((thread = id), (comments = true))} />
	</div>

	<div class="pointer-events-none absolute inset-0 flex flex-col justify-between gap-3 p-3">
		<div class="flex items-start justify-between gap-2">
			<div class={chrome} data-board-ui>
				{#if back}
					<IconButton variant="ghost" aria-label="Retour aux tableaux" onclick={() => back && goto(back)}>
						<Icon icon={icons.chevronLeft} size={20} />
					</IconButton>
				{/if}
				{#if editingTitle}
					<input
						bind:this={titleInput}
						bind:value={currentTitle}
						class="max-w-[40vw] rounded-fc-xs bg-fc-surface px-2 py-0.5 text-fc-sm font-semibold text-fc-fg focus:outline-2 focus:outline-fc-ring"
						aria-label="Nom du tableau"
						onblur={saveTitle}
						onkeydown={(e) => {
							if (e.key === 'Enter') void saveTitle();
							else if (e.key === 'Escape') (currentTitle = name), (editingTitle = false);
						}}
					/>
				{:else}
					<button
						type="button"
						class="max-w-[40vw] truncate rounded-fc-xs px-2 py-0.5 text-left text-fc-sm font-semibold text-fc-fg hover:bg-fc-surface/60 focus-visible:outline-2 focus-visible:outline-fc-ring disabled:cursor-default disabled:hover:bg-transparent"
						disabled={!canEdit(session.role)}
						title={canEdit(session.role) ? 'Cliquer pour renommer' : name}
						onclick={startEditTitle}
					>
						{name}
					</button>
				{/if}
				<span class="flex items-center gap-1.5 pr-3 text-fc-xs text-fc-fg-muted" aria-live="polite">
					{#if session.connection === 'connecting'}<Spinner size="sm" />{/if}
					{STATUS[session.connection]}
				</span>
			</div>
			<div class="flex items-center gap-2">
				<AvatarStack {session} />
				<div class={chrome} data-board-ui>
					<Tooltip text="Rechercher (⌘ K)">
						<IconButton variant="ghost" aria-label="Rechercher" onclick={() => (search = true)}
							><Icon icon={icons.search} size={20} /></IconButton
						>
					</Tooltip>
					<Tooltip text="Commentaires">
						<IconButton
							variant="ghost"
							aria-label="Commentaires"
							aria-pressed={comments}
							onclick={() => ((comments = !comments), (summary = null))}
						>
							<Icon icon={toolIcons.comment} size={20} />
						</IconButton>
					</Tooltip>
					{#if aiBoard}
						<AiMenu board={aiBoard} {editor} bind:busy={thinking} bind:limit onsummarize={summarize} />
					{/if}
					<span class="hidden md:inline-flex">
						<Tooltip text="Raccourcis (?)">
							<IconButton variant="ghost" aria-label="Raccourcis clavier" onclick={() => (help = true)}
								><Icon icon={toolIcons.keyboard} size={20} /></IconButton
							>
						</Tooltip>
					</span>
					{#if session.role === 'owner' && 'board' in session.target}
						<Tooltip text="Partager">
							<IconButton variant="ghost" aria-label="Partager" onclick={() => (sharing = true)}
								><Icon icon={toolIcons.share} size={20} /></IconButton
							>
						</Tooltip>
					{/if}
				</div>
			</div>
		</div>

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
			{#if summary !== null}
				<div class="absolute right-3 bottom-24 left-3 md:static md:h-full">
					<AiPanel markdown={summary} busy={thinking} onretry={summarize} onclose={() => (summary = null)} />
				</div>
			{:else if comments}
				<div class="absolute right-3 bottom-24 left-3 md:static md:h-full">
					<CommentPanel {session} bind:active={thread} onclose={() => (comments = false)} />
				</div>
			{/if}
		</div>

		<div class="flex flex-col items-center gap-2 md:flex-row md:items-end md:justify-between">
			<div class="hidden md:block md:w-48"></div>
			<div class="flex flex-col items-center gap-2">
				<StyleBar {editor} {ui} />
				<Toolbar {editor} onimage={() => picker.click()} />
			</div>
			<div class="flex flex-col items-end gap-2">
				{#if minimap}<span class="hidden lg:block"><Minimap {editor} /></span>{/if}
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
