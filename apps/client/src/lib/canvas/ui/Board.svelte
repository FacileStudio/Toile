<script lang="ts">
	import { onMount } from 'svelte';
	import ContextMenu, { type ContextMenuItem } from '$lib/components/ContextMenu.svelte';
	import type { Editor } from '../editor.svelte';
	import { Input, isTyping } from '../input';
	import { commandFor } from '../shortcuts';
	import { runCommand, type CommandUi } from '../commands';
	import { pasteInto, copyOut } from '../paste';
	import { watchTheme } from '../render/palette';
	import { topHit } from '../hit';
	import { canvasMenuItems, elementMenuItems } from '../contextmenu';
	import SelectionHandles from './SelectionHandles.svelte';
	import TextEditor from './TextEditor.svelte';

	let { editor, ui, input = $bindable() }: { editor: Editor; ui: CommandUi; input?: Input } = $props();

	let host: HTMLDivElement;
	let scene: HTMLCanvasElement;
	let overlay: HTMLCanvasElement;
	let cursor = $state('default');
	let contextMenu = $state<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

	onMount(() => {
		const control = new Input(editor, host);
		input = control;
		editor.attach(scene, overlay);
		editor.setCamera({ x: host.clientWidth / 2, y: host.clientHeight / 2, z: 1 }, true);
		const stop = control.listen();
		const resize = new ResizeObserver(() => editor.resize());
		resize.observe(host);
		const unwatch = watchTheme(() => editor.retheme());
		return () => {
			stop();
			resize.disconnect();
			unwatch();
		};
	});

	$effect(() => {
		input?.setTool(editor.tool);
	});

	function keydown(event: KeyboardEvent) {
		if (isTyping(event.target) || editor.editing) return;
		const command = commandFor(event);
		if (command && runCommand(editor, command, ui)) event.preventDefault();
	}

	function clip(event: ClipboardEvent, cut: boolean) {
		if (isTyping(event.target) || editor.editing || !event.clipboardData) return;
		if (copyOut(editor, event.clipboardData, cut)) event.preventDefault();
	}

	function paste(event: ClipboardEvent) {
		if (isTyping(event.target) || editor.editing || !event.clipboardData) return;
		event.preventDefault();
		void pasteInto(editor, event.clipboardData, null);
	}

	function drop(event: DragEvent) {
		event.preventDefault();
		if (!event.dataTransfer) return;
		const rect = host.getBoundingClientRect();
		void pasteInto(
			editor,
			event.dataTransfer,
			editor.world({ x: event.clientX - rect.left, y: event.clientY - rect.top })
		);
	}

	function handleContextMenu(event: MouseEvent) {
		event.preventDefault();
		if (editor.editing) editor.finishEditing();
		const rect = host.getBoundingClientRect();
		const world = editor.world({ x: event.clientX - rect.left, y: event.clientY - rect.top });
		const hit = topHit(editor.scene, world, 4 / editor.camera.z);

		if (hit) {
			if (!editor.selection.includes(hit.id)) editor.select([hit.id]);
			contextMenu = { x: event.clientX, y: event.clientY, items: elementMenuItems(editor, ui, hit) };
			return;
		}

		editor.select([]);
		contextMenu = { x: event.clientX, y: event.clientY, items: canvasMenuItems(editor, ui, world) };
	}
</script>

<svelte:window onkeydown={keydown} oncopy={(e) => clip(e, false)} oncut={(e) => clip(e, true)} onpaste={paste} />

<div
	bind:this={host}
	class="relative h-full w-full touch-none overflow-hidden select-none"
	style:cursor
	role="application"
	aria-label="Tableau"
	aria-roledescription="tableau blanc"
	onpointermove={() => (cursor = input?.cursor ?? 'default')}
	onscroll={() => host.scrollTo(0, 0)}
	ondragover={(e) => e.preventDefault()}
	ondrop={drop}
	oncontextmenu={handleContextMenu}
>
	<canvas bind:this={scene} class="absolute inset-0 h-full w-full" data-testid="board-canvas"></canvas>
	<canvas bind:this={overlay} class="pointer-events-none absolute inset-0 h-full w-full"></canvas>
	{#if input}
		<SelectionHandles {editor} {input} />
	{/if}
	<TextEditor {editor} />
	{#if contextMenu}
		<ContextMenu x={contextMenu.x} y={contextMenu.y} items={contextMenu.items} onclose={() => (contextMenu = null)} />
	{/if}
</div>
