<script lang="ts">
	import { untrack } from 'svelte';
	import type * as Y from 'yjs';
	import { LOCAL } from '$lib/board/doc';
	import { LIMITS } from '$lib/board/schema';
	import { setText } from '$lib/board/text';
	import type { Editor } from '../editor.svelte';
	import { toScreen } from '../camera';
	import { inkOn, STICKY_FILL } from '../render/palette';
	import { fontOf, fontSize, textHeight, textPadding } from '../render/text';
	import { FRAME_LABEL } from '../hit';

	let { editor }: { editor: Editor } = $props();

	let field = $state<HTMLTextAreaElement | HTMLInputElement | null>(null);
	let measure: CanvasRenderingContext2D | null = null;

	const shape = $derived.by(() => {
		void editor.version;
		return editor.editing ? (editor.scene.get(editor.editing) ?? null) : null;
	});

	const view = $derived.by(() => {
		if (!shape) return null;
		const c = editor.camera;
		const at = toScreen(c, { x: shape.x, y: shape.type === 'frame' ? shape.y - FRAME_LABEL : shape.y });
		const fill = shape.type === 'sticky' ? (shape.style.fill ?? STICKY_FILL) : shape.style.fill;
		const color =
			shape.type === 'text' ? (shape.style.stroke ?? 'var(--color-fc-fg)') : fill ? inkOn(fill) : 'var(--color-fc-fg)';
		return { x: at.x, y: at.y, z: c.z, color };
	});

	$effect(() => {
		const id = editor.editing;
		const text = id ? editor.doc.text(id) : null;
		if (!field || !id) return;
		const target = field;
		untrack(() => {
			const current = editor.scene.get(id);
			target.value = current?.type === 'frame' ? (current.name ?? '') : (text?.toString() ?? '');
		});
		target.focus({ preventScroll: true });
		target.select();
		if (!text) return;
		const sync = (_event: Y.YTextEvent, tr: Y.Transaction) => {
			if (tr.origin === LOCAL || !field) return;
			const [start, end] = [field.selectionStart, field.selectionEnd];
			field.value = text.toString();
			field.setSelectionRange(start, end);
		};
		text.observe(sync);
		return () => text.unobserve(sync);
	});

	function oninput() {
		const id = editor.editing;
		if (!id || !field || !shape) return;
		if (shape.type === 'frame') return editor.doc.update(id, { name: field.value.slice(0, 200) });
		const text = editor.doc.text(id);
		if (!text) return;
		const value = field.value;
		editor.doc.transact(() => {
			setText(text, value);
			if (shape.type !== 'text') return;
			measure ??= document.createElement('canvas').getContext('2d');
			if (measure) editor.doc.update(id, { h: textHeight(measure, { ...shape, text: value }, editor.palette.font) });
		});
	}

	function finish() {
		editor.finishEditing();
	}

	function keydown(event: KeyboardEvent) {
		if (
			event.key === 'Escape' ||
			(event.key === 'Enter' && (event.metaKey || event.ctrlKey || shape?.type === 'frame'))
		) {
			event.preventDefault();
			finish();
		}
	}
</script>

{#if shape && view}
	{#if shape.type === 'frame'}
		<input
			bind:this={field}
			class="absolute top-0 left-0 origin-top-left rounded-fc-xs bg-fc-surface px-1 text-fc-fg-muted outline-none"
			style:transform="translate({view.x}px, {view.y}px)"
			style:font="500 14px var(--font-fc-body)"
			style:width="{Math.max(160, shape.w * view.z)}px"
			aria-label="Nom du cadre"
			maxlength={200}
			{oninput}
			onblur={finish}
			onkeydown={keydown}
			data-board-ui
		/>
	{:else}
		<textarea
			bind:this={field}
			class="absolute top-0 left-0 origin-top-left resize-none overflow-hidden bg-transparent outline-none"
			style:transform="translate({view.x}px, {view.y}px) scale({view.z}) translate({shape.w / 2}px, {shape.h / 2}px)
			rotate({shape.rot}rad) translate({-shape.w / 2}px, {-shape.h / 2}px)"
			style:width="{shape.w}px"
			style:height="{shape.h}px"
			style:padding="{shape.type === 'text'
				? textPadding(shape)
				: Math.max(textPadding(shape), (shape.h - fontSize(shape) * 1.3) / 2 - 1)}px {textPadding(shape)}px"
			style:font={fontOf(shape, 'var(--font-fc-body)')}
			style:line-height="1.3"
			style:color={view.color}
			style:text-align={shape.style.align ?? (shape.type === 'text' ? 'left' : 'center')}
			aria-label="Texte"
			maxlength={LIMITS.textChars}
			{oninput}
			onblur={finish}
			onkeydown={keydown}
			data-board-ui
		></textarea>
	{/if}
{/if}
