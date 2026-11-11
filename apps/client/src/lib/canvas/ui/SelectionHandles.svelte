<script lang="ts">
	import type { Editor } from '../editor.svelte';
	import type { Input } from '../input';
	import { toScreen } from '../camera';
	import { rotatedBox, union } from '../geometry';
	import { HANDLES, type Handle } from '../transform';
	import { ResizeGesture, RotateGesture } from '../tools/handles';
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
	const CURSOR: Record<Handle, string> = {
		nw: 'nwse-resize',
		se: 'nwse-resize',
		ne: 'nesw-resize',
		sw: 'nesw-resize',
		n: 'ns-resize',
		s: 'ns-resize',
		e: 'ew-resize',
		w: 'ew-resize'
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

	function point(event: PointerEvent) {
		const rect = (event.currentTarget as HTMLElement).closest('[role="application"]')?.getBoundingClientRect();
		return editor.world({ x: event.clientX - (rect?.left ?? 0), y: event.clientY - (rect?.top ?? 0) });
	}

	function resize(handle: Handle, event: PointerEvent) {
		event.stopPropagation();
		input.startGesture(new ResizeGesture(editor, handle, point(event)), event);
	}

	function spin(event: PointerEvent) {
		event.stopPropagation();
		input.startGesture(new RotateGesture(editor, point(event)), event);
	}

	const small = $derived(frame ? Math.min(frame.w, frame.h) < 28 : false);
</script>

{#if frame && editor.editable && !frame.locked}
	<div
		class="pointer-events-none absolute top-0 left-0 origin-center"
		style:width="{frame.w}px"
		style:height="{frame.h}px"
		style:transform="translate({frame.x}px, {frame.y}px) rotate({frame.rot}rad)"
		data-board-ui
	>
		{#each HANDLES as handle (handle)}
			{#if !small || handle.length === 2}
				<button
					type="button"
					class="handle pointer-events-auto absolute flex -translate-x-1/2 -translate-y-1/2 touch-none items-center justify-center"
					style:left="{POS[handle][0] * 100}%"
					style:top="{POS[handle][1] * 100}%"
					style:cursor={CURSOR[handle]}
					aria-label="Redimensionner ({LABEL[handle]})"
					onpointerdown={(e) => resize(handle, e)}
				>
					<span class="size-2.5 rounded-fc-xs bg-fc-page shadow-sm outline-2 outline-fc-info"></span>
				</button>
			{/if}
		{/each}
		<button
			type="button"
			class="handle pointer-events-auto absolute -top-9 left-1/2 flex -translate-x-1/2 -translate-y-1/2 touch-none items-center justify-center"
			style:cursor="grab"
			aria-label="Faire pivoter"
			onpointerdown={spin}
		>
			<span class="size-3 rounded-fc-pill bg-fc-page shadow-sm outline-2 outline-fc-info"></span>
		</button>
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
			<span class="size-3 rounded-fc-pill bg-fc-page shadow-sm outline-2 outline-fc-info"></span>
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
