<script lang="ts">
	import { Icon } from '@facile/muse';
	import { Bar, Cursor, Stage, Window, type Act, type CursorState } from '@facile/mise-en-scene';
	import LiveCursor from './LiveCursor.svelte';
	import Sticky from './Sticky.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { perform } from './collab';
	import { COLORS, COMPACT, TOOLS, WIDE, blank, clear, pose, posed, type BoardNodes, type HeroLayout, type StickyId, type Who } from './hero';

	const frames = $derived([t('hero.ideas'), t('hero.todo')]);
	const INK: Record<Who, string> = { you: 'var(--color-fc-info)', lea: COLORS.lea, malik: COLORS.malik };

	let board = $state(blank());
	let still = $state(false);
	let nodes = $state<BoardNodes>({ stickies: {}, tool: () => undefined });

	nodes.tool = (id) => nodes.canvas?.querySelector(`[data-tool="${id}"]`) ?? undefined;

	function owner(id: StickyId): Who | undefined {
		return (Object.keys(board.picks) as Who[]).find((who) => board.picks[who] === id);
	}

	function ink(id: StickyId): string | null {
		const who = owner(id);
		return who ? INK[who] : null;
	}

	function shown(who: Who, current: HeroLayout): CursorState {
		return still ? { ...pose(current)[who], visible: true, down: who !== 'malik' } : board.cursors[who];
	}

	function reset() {
		still = false;
		clear(board);
	}

	function poster() {
		still = true;
		posed(board);
	}

	async function script(act: Act) {
		await perform(act, board, nodes, (nodes.canvas?.offsetWidth ?? 0) < 500 ? COMPACT : WIDE);
	}
</script>

<Stage
	width={960}
	height={640}
	compact={{ below: 640, width: 358, height: 620 }}
	{script}
	{reset}
	{poster}
	label={t('hero.label')}
>
	{#snippet children({ compact })}
		{@const current = compact ? COMPACT : WIDE}
		<Window title={compact ? undefined : t('hero.window')} style={current.window}>
			{#snippet trailing()}
				<span class="flex items-center gap-2">
					<span class="flex">
						{#each [COLORS.lea, COLORS.malik] as color, at (at)}
							<span class="size-5 rounded-fc-full {at > 0 ? '-ms-1.5' : ''}" style:background={color}></span>
						{/each}
					</span>
					<span class="rounded-fc-pill bg-fc-accent px-2.5 py-1 text-fc-xs font-medium text-fc-accent-fg">{t('hero.share')}</span>
				</span>
			{/snippet}
			<div bind:this={nodes.canvas} class="board relative flex-1 overflow-hidden">
				{#each current.frames as frame, at (at)}
					<div
						class="absolute rounded-fc-lg bg-fc-surface"
						style:left="{frame.x}px"
						style:top="{frame.y}px"
						style:width="{frame.w}px"
						style:height="{frame.h}px"
					>
						<span class="absolute -top-5 left-1 text-fc-xs font-medium text-fc-fg-muted">{frames[at]}</span>
					</div>
				{/each}
				{#each current.extra as box, at (at)}
					<Sticky {box} color={COLORS.extra[at]}><Bar w="75%" /><Bar w="50%" /><Bar w="62%" /></Sticky>
				{/each}
				<Sticky
					box={current.export}
					color={COLORS.export}
					selected={ink('export')}
					own={owner('export') === 'you'}
					bind:node={nodes.stickies.export}
				>
					{t('hero.export')}<span class="min-h-5">{board.line}</span><Bar w="45%" />
				</Sticky>
				<Sticky
					box={current.beta}
					color={COLORS.beta}
					shown={board.placed}
					selected={ink('beta')}
					own={owner('beta') === 'you'}
					bind:node={nodes.stickies.beta}
				>
					<span class="min-h-5">{board.beta}</span><span class="min-h-5">{board.note}</span>
				</Sticky>
				<Sticky
					box={current.onboarding}
					color={COLORS.ideas}
					shift={board.moved ? current.drag : undefined}
					selected={ink('onboarding')}
					own={owner('onboarding') === 'you'}
					bind:node={nodes.stickies.onboarding}
				>
					{t('hero.onboarding')}<Bar w="80%" /><Bar w="55%" />
				</Sticky>
				<svg class="pointer-events-none absolute inset-0 size-full" aria-hidden="true">
					<path d={current.arrow} pathLength="1" class="arrow" class:drawn={board.linked} />
					<path d={current.head} class="head" class:drawn={board.linked} />
				</svg>
				<div
					class="toolbar absolute flex gap-1 rounded-fc-pill bg-fc-component p-1 shadow-sm"
					class:column={current.toolbar === 'column'}
				>
					{#each TOOLS as item (item.id)}
						<span
							class="flex size-9 items-center justify-center rounded-fc-full {board.tool === item.id
								? 'bg-fc-accent text-fc-accent-fg'
								: 'text-fc-fg-muted'}"
							data-tool={item.id}
						>
							<Icon icon={item.icon} size={18} />
						</span>
					{/each}
				</div>
			</div>
		</Window>
		<LiveCursor cursor={shown('lea', current)} color={COLORS.lea} name="Léa" />
		<LiveCursor cursor={shown('malik', current)} color={COLORS.malik} name="Malik" />
		<Cursor cursor={shown('you', current)} />
	{/snippet}
</Stage>

<style>
	.board {
		background-image: radial-gradient(color-mix(in oklch, var(--color-fc-fg) 16%, transparent) 1px, transparent 1px);
		background-size: 22px 22px;
	}
	.arrow {
		fill: none;
		stroke: var(--color-fc-fg-muted);
		stroke-width: 2;
		stroke-linecap: round;
		stroke-dasharray: 1;
		stroke-dashoffset: 1;
		opacity: 0;
		transition:
			stroke-dashoffset 0.9s cubic-bezier(0.45, 0, 0.1, 1),
			opacity 0.3s ease;
	}
	.head {
		fill: none;
		stroke: var(--color-fc-fg-muted);
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
		opacity: 0;
		transition: opacity 0.2s ease;
	}
	.drawn {
		stroke-dashoffset: 0;
		opacity: 1;
	}
	.head.drawn {
		transition-delay: 0.75s;
	}
	.toolbar {
		left: 50%;
		bottom: 14px;
		translate: -50% 0;
	}
	.column {
		flex-direction: column;
		left: 16px;
		top: 50%;
		bottom: auto;
		translate: 0 -50%;
	}
</style>
