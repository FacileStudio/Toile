<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { Editor } from '../editor.svelte';
	import { viewport, type Box } from '../camera';
	import { union } from '../geometry';
	import { fitCanvas } from '../render/renderer';

	let { editor }: { editor: Editor } = $props();

	const W = 200;
	const H = 140;
	let canvas: HTMLCanvasElement;
	let world: Box | null = null;
	let timer = 0;

	function area(): Box | null {
		const view = viewport(editor.camera, editor.size.width, editor.size.height);
		const boxes: Box[] = [view];
		for (const id of editor.scene.shapes.keys()) {
			const box = editor.scene.index.box(id);
			if (box) boxes.push(box);
		}
		return union(boxes);
	}

	function draw() {
		timer = 0;
		const ctx = canvas?.getContext('2d');
		const box = area();
		if (!ctx || !box) return;
		const { dpr } = fitCanvas(canvas);
		const scale = Math.min(W / box.w, H / box.h);
		world = { x: box.x - (W / scale - box.w) / 2, y: box.y - (H / scale - box.h) / 2, w: W / scale, h: H / scale };
		ctx.setTransform(dpr * scale, 0, 0, dpr * scale, -world.x * dpr * scale, -world.y * dpr * scale);
		ctx.clearRect(world.x, world.y, world.w, world.h);
		for (const shape of editor.scene.order) {
			ctx.fillStyle = shape.style.fill ?? editor.palette.muted;
			const b = editor.scene.index.box(shape.id);
			if (b) ctx.fillRect(b.x, b.y, Math.max(b.w, 1 / scale), Math.max(b.h, 1 / scale));
		}
		const view = viewport(editor.camera, editor.size.width, editor.size.height);
		ctx.strokeStyle = editor.palette.info;
		ctx.lineWidth = 1.5 / scale;
		ctx.strokeRect(view.x, view.y, view.w, view.h);
	}

	$effect(() => {
		void editor.version;
		void editor.camera;
		void editor.size;
		if (timer) window.clearTimeout(timer);
		timer = window.setTimeout(draw, 160);
	});

	onDestroy(() => window.clearTimeout(timer));

	function jump(event: PointerEvent) {
		if (!world || (event.type === 'pointermove' && !event.buttons)) return;
		const rect = canvas.getBoundingClientRect();
		const x = world.x + ((event.clientX - rect.left) / W) * world.w;
		const y = world.y + ((event.clientY - rect.top) / H) * world.h;
		const z = editor.goal.z;
		const camera = { x: editor.size.width / 2 - x * z, y: editor.size.height / 2 - y * z, z };
		if (event.type === 'pointerdown') editor.glide(camera);
		else editor.setCamera(camera);
	}
</script>

<canvas
	bind:this={canvas}
	class="pointer-events-auto touch-none rounded-fc-lg bg-fc-component shadow-lg"
	style:width="{W}px"
	style:height="{H}px"
	aria-hidden="true"
	onpointerdown={(e) => {
		(e.currentTarget as HTMLCanvasElement).setPointerCapture(e.pointerId);
		jump(e);
	}}
	onpointermove={jump}
	data-board-ui
></canvas>
