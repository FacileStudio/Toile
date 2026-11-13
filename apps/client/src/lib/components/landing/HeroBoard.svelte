<script lang="ts">
	import { Icon } from '@facile/muse';
	import { Bar, Cursor, Stage, Window, cursorAt, type Act } from '@facile/mise-en-scene';
	import LiveCursor from './LiveCursor.svelte';
	import Sticky from './Sticky.svelte';
	import { COLORS, COMPACT, TOOLS, WIDE } from './hero';

	const BETA = 'Lancer la bêta';
	const FRAMES = ['Idées', 'À faire'];

	let tool = $state<string>('select');
	let placed = $state(false);
	let text = $state('');
	let linked = $state(false);
	let lea = $state(0);
	let malik = $state(0);
	let cursor = $state(cursorAt());
	let canvas = $state<HTMLElement>();
	let beta = $state<HTMLElement>();

	function reset() {
		tool = 'select';
		placed = false;
		text = '';
		linked = false;
		lea = 0;
		malik = 0;
		cursor.visible = false;
	}

	function poster() {
		placed = true;
		text = BETA;
		linked = true;
		lea = 1;
		malik = 2;
	}

	async function script(act: Act) {
		Object.assign(cursor, act.point(canvas, 0.86, 0.9));
		await act.wait(700);
		cursor.visible = true;
		lea = 1;
		const stickyTool = canvas?.querySelector('[data-tool="sticky"]') ?? undefined;
		await act.move(cursor, act.point(stickyTool, 0.6, 0.6));
		await act.click(cursor, () => (tool = 'sticky'));
		await act.move(cursor, act.point(beta, 0.35, 0.4));
		await act.click(cursor, () => (placed = true));
		tool = 'select';
		await act.wait(250);
		await act.type(BETA, (value) => (text = value), 55);
		await act.move(cursor, act.point(beta, 0.9, 1.25), 500);
		malik = 1;
		await act.wait(950);
		linked = true;
		malik = 2;
		await act.wait(2600);
		placed = false;
		linked = false;
		cursor.visible = false;
		await act.wait(700);
	}
</script>

<Stage
	width={960}
	height={640}
	compact={{ below: 640, width: 358, height: 620 }}
	{script}
	{reset}
	{poster}
	label="Démonstration : trois personnes ajoutent des post-it et une flèche sur le même tableau, en direct."
>
	{#snippet children({ compact })}
		{@const layout = compact ? COMPACT : WIDE}
		<Window title={compact ? undefined : 'Rétro du sprint'} style={layout.window}>
			{#snippet trailing()}
				<span class="flex items-center gap-2">
					<span class="flex gap-1">
						{#each [COLORS.lea, COLORS.malik, COLORS.beta] as color, at (at)}
							<span class="size-5 rounded-fc-full" style:background={color}></span>
						{/each}
					</span>
					<span class="rounded-fc-pill bg-fc-accent px-2.5 py-1 text-fc-xs font-medium text-fc-accent-fg">Partager</span>
				</span>
			{/snippet}
			<div bind:this={canvas} class="board relative flex-1 overflow-hidden">
				{#each layout.frames as frame, at (at)}
					<div
						class="absolute rounded-fc-lg bg-fc-surface"
						style:left="{frame.x}px"
						style:top="{frame.y}px"
						style:width="{frame.w}px"
						style:height="{frame.h}px"
					>
						<span class="absolute -top-5 left-1 text-fc-xs font-medium text-fc-fg-muted">{FRAMES[at]}</span>
					</div>
				{/each}
				{#each layout.extra as box, at (at)}
					<Sticky {box} color={COLORS.extra[at]}><Bar w="75%" /><Bar w="50%" /><Bar w="62%" /></Sticky>
				{/each}
				<Sticky box={layout.stickies[0]} color={COLORS.ideas}>
					Onboarding<Bar w="80%" /><Bar w="55%" />
				</Sticky>
				<Sticky box={layout.stickies[1]} color={COLORS.export}>
					Export PDF<Bar w="70%" /><Bar w="45%" />
				</Sticky>
				<Sticky box={layout.stickies[2]} color={COLORS.beta} shown={placed} bind:node={beta}>
					<span class="min-h-5">{text}</span>
				</Sticky>
				<svg class="pointer-events-none absolute inset-0 size-full" aria-hidden="true">
					<path d={layout.arrow} pathLength="1" class="arrow" class:drawn={linked} />
					<path d={layout.head} class="head" class:drawn={linked} />
				</svg>
				<div
					class="toolbar absolute flex gap-1 rounded-fc-pill bg-fc-component p-1 shadow-sm"
					class:column={layout.toolbar === 'column'}
				>
					{#each TOOLS as item (item.id)}
						<span
							class="flex size-9 items-center justify-center rounded-fc-full {tool === item.id
								? 'bg-fc-accent text-fc-accent-fg'
								: 'text-fc-fg-muted'}"
							data-tool={item.id}
						>
							<Icon icon={item.icon} size={18} />
						</span>
					{/each}
				</div>
				<LiveCursor {...layout.lea[lea]} color={COLORS.lea} name="Léa" />
				<LiveCursor {...layout.malik[malik]} color={COLORS.malik} name="Malik" />
			</div>
		</Window>
		<Cursor {cursor} />
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
