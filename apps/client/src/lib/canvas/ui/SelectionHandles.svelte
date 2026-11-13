<script lang="ts">
	import type { Editor } from '../editor.svelte';
	import type { Input } from '../input';
	import { anchorPoint } from '../anchors';
	import { toScreen } from '../camera';
	import { rotatedBox, union } from '../geometry';
	import { CORNERS, type Handle } from '../transform';
	import { ResizeGesture } from '../tools/handles';
	import { ConnectorTool } from '../tools/connector';
	import { openCard } from '../tools/select';
	import { EndpointGesture } from '../tools/endpoint';
	import { pathPoints } from '../bounds';

	let { editor, input }: { editor: Editor; input: Input } = $props();

	const ends = $derived.by(() => {
		void editor.version;
		const only = editor.selection.length === 1 ? editor.scene.get(editor.selection[0]) : null;
		if (!only || (only.type !== 'arrow' && only.type !== 'line') || only.locked || editor.tool !== 'select')
			return null;
		const points = pathPoints(only, editor.scene.get).map((p) => toScreen(editor.camera, p));
		return points.length < 2 ? null : { id: only.id, from: points[0], to: points[points.length - 1] };
	});

	function drag(end: 'from' | 'to', event: PointerEvent) {
		if (!ends) return;
		event.stopPropagation();
		input.startGesture(new EndpointGesture(editor, ends.id, end), event);
	}

	const frame = $derived.by(() => {
		void editor.version;
		const shapes = editor.selection.map((id) => editor.scene.get(id)).filter((s) => !!s);
		if (!shapes.length || editor.editing || editor.tool !== 'select') return null;
		if (shapes.length === 1 && (shapes[0].type === 'arrow' || shapes[0].type === 'line')) return null;
		const single = shapes.length === 1 ? shapes[0] : null;
		const box = single ?? union(shapes.map((s) => rotatedBox(s)));
		if (!box) return null;
		const c = editor.camera;
		const at = toScreen(c, { x: box.x, y: box.y });
		const locked = shapes.every((s) => s.locked);
		return { x: at.x, y: at.y, w: box.w * c.z, h: box.h * c.z, rot: single?.rot ?? 0, locked };
	});

	const single = $derived.by(() => {
		void editor.version;
		return editor.selection.length === 1 ? editor.scene.get(editor.selection[0]) : null;
	});

	const POS: Record<Handle, [number, number]> = {
		nw: [0, 0],
		n: [0.5, 0],
		ne: [1, 0],
		e: [1, 0.5],
		se: [1, 1],
		s: [0.5, 1],
		sw: [0, 1],
		w: [0, 0.5]
	};
	const LABEL: Record<Handle, string> = {
		nw: 'haut gauche',
		n: 'haut',
		ne: 'haut droite',
		e: 'droite',
		se: 'bas droite',
		s: 'bas',
		sw: 'bas gauche',
		w: 'gauche'
	};

	function handleCursor(handle: Handle, rot = 0): string {
		const base: Record<Handle, number> = {
			n: 0,
			ne: 45,
			e: 90,
			se: 135,
			s: 180,
			sw: 225,
			w: 270,
			nw: 315
		};
		const deg = (((base[handle] + (rot * 180) / Math.PI) % 360) + 360) % 360;
		if (deg >= 22.5 && deg < 67.5) return 'nesw-resize';
		if (deg >= 67.5 && deg < 112.5) return 'ew-resize';
		if (deg >= 112.5 && deg < 157.5) return 'nwse-resize';
		if (deg >= 157.5 && deg < 202.5) return 'ns-resize';
		if (deg >= 202.5 && deg < 247.5) return 'nesw-resize';
		if (deg >= 247.5 && deg < 292.5) return 'ew-resize';
		if (deg >= 292.5 && deg < 337.5) return 'nwse-resize';
		return 'ns-resize';
	}

	const OUTWARD: { id: string; label: string; left: string; top: string; anchor: [number, number] }[] = [
		{ id: 'n', label: 'Connecter (haut)', left: '50%', top: '-22px', anchor: [0.5, 0] },
		{ id: 'e', label: 'Connecter (droite)', left: 'calc(100% + 22px)', top: '50%', anchor: [1, 0.5] },
		{ id: 's', label: 'Connecter (bas)', left: '50%', top: 'calc(100% + 22px)', anchor: [0.5, 1] },
		{ id: 'w', label: 'Connecter (gauche)', left: '-22px', top: '50%', anchor: [0, 0.5] }
	];

	function point(event: PointerEvent) {
		const rect = (event.currentTarget as HTMLElement).closest('[role="application"]')?.getBoundingClientRect();
		return editor.world({ x: event.clientX - (rect?.left ?? 0), y: event.clientY - (rect?.top ?? 0) });
	}

	function resize(handle: Handle, event: PointerEvent) {
		event.stopPropagation();
		input.startGesture(new ResizeGesture(editor, handle, point(event)), event);
	}

	function connect(anchor: [number, number], event: PointerEvent) {
		const shape = editor.selection.length === 1 ? editor.scene.get(editor.selection[0]) : null;
		if (!shape) return;
		event.stopPropagation();
		const p = anchorPoint(shape, anchor);
		const tool = new ConnectorTool(editor, 'arrow', { point: p, binding: { shape: shape.id, anchor } });
		input.startGesture(tool, event);
	}
</script>

{#if frame && editor.editable && !frame.locked}
	<div
		class="pointer-events-none absolute top-0 left-0 origin-center"
		style:width="{frame.w}px"
		style:height="{frame.h}px"
		style:transform="translate({frame.x}px, {frame.y}px) rotate({frame.rot}rad)"
		data-board-ui
	>
		<button
			type="button"
			class="pointer-events-auto absolute -top-1.5 left-2 right-2 h-3 touch-none"
			style:cursor={handleCursor('n', frame.rot)}
			aria-label="Redimensionner ({LABEL.n})"
			onpointerdown={(e) => resize('n', e)}
		></button>
		<button
			type="button"
			class="pointer-events-auto absolute -bottom-1.5 left-2 right-2 h-3 touch-none"
			style:cursor={handleCursor('s', frame.rot)}
			aria-label="Redimensionner ({LABEL.s})"
			onpointerdown={(e) => resize('s', e)}
		></button>
		<button
			type="button"
			class="pointer-events-auto absolute top-2 bottom-2 -left-1.5 w-3 touch-none"
			style:cursor={handleCursor('w', frame.rot)}
			aria-label="Redimensionner ({LABEL.w})"
			onpointerdown={(e) => resize('w', e)}
		></button>
		<button
			type="button"
			class="pointer-events-auto absolute top-2 bottom-2 -right-1.5 w-3 touch-none"
			style:cursor={handleCursor('e', frame.rot)}
			aria-label="Redimensionner ({LABEL.e})"
			onpointerdown={(e) => resize('e', e)}
		></button>
		{#each CORNERS as handle (handle)}
			<button
				type="button"
				class="handle pointer-events-auto absolute flex -translate-x-1/2 -translate-y-1/2 touch-none items-center justify-center"
				style:left="{POS[handle][0] * 100}%"
				style:top="{POS[handle][1] * 100}%"
				style:cursor={handleCursor(handle, frame.rot)}
				aria-label="Redimensionner ({LABEL[handle]})"
				onpointerdown={(e) => resize(handle, e)}
			>
				<span class="size-3 rounded-[3px] bg-fc-info shadow-sm"></span>
			</button>
		{/each}
		{#if single && !frame.locked}
			{#each OUTWARD as c (c.id)}
				<button
					type="button"
					class="handle pointer-events-auto absolute flex -translate-x-1/2 -translate-y-1/2 touch-none items-center justify-center group"
					style:left={c.left}
					style:top={c.top}
					style:cursor="crosshair"
					aria-label={c.label}
					onpointerdown={(e) => connect(c.anchor, e)}
				>
					<span
						class="size-3 rounded-fc-pill bg-fc-info shadow-sm transition-transform group-hover:scale-125 motion-reduce:transition-none"
					></span>
				</button>
			{/each}
		{/if}
		{#if single?.type === 'card' && single.url}
			{@const topH = (single.image ? Math.min(single.h * 0.6, 120) : 0) * editor.camera.z}
			{#if single.kind === 'youtube'}
				<button
					type="button"
					class="pointer-events-auto absolute left-1/2 flex -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-fc-pill transition-transform hover:scale-110 active:scale-95 cursor-pointer z-10"
					style:top="{topH / 2}px"
					style:width="{Math.min(48, topH * 0.6)}px"
					style:height="{Math.min(48, topH * 0.6)}px"
					aria-label="Lire la vidéo"
					title="Lire la vidéo"
					onclick={(e) => {
						e.stopPropagation();
						openCard(editor, single);
					}}
				>
					<span class="sr-only">Lire la vidéo</span>
				</button>
			{/if}

		{/if}
	</div>
{/if}

{#if ends && editor.editable}
	{#each ['from', 'to'] as const as end (end)}
		<button
			type="button"
			class="handle absolute top-0 left-0 flex touch-none items-center justify-center"
			style:transform="translate({ends[end].x}px, {ends[end].y}px) translate(-50%, -50%)"
			style:cursor="crosshair"
			aria-label={end === 'from' ? 'Déplacer le début de la flèche' : 'Déplacer la fin de la flèche'}
			onpointerdown={(e) => drag(end, e)}
			data-board-ui
		>
			<span class="size-3 rounded-fc-pill bg-fc-info shadow-sm"></span>
		</button>
	{/each}
{/if}

<style>
	.handle {
		width: 24px;
		height: 24px;
	}

	@media (pointer: coarse) {
		.handle {
			width: 44px;
			height: 44px;
		}
	}
</style>
