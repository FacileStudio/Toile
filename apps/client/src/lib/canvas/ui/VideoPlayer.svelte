<script lang="ts">
	import { Icon, IconButton, icons } from '@facile/muse';
	import type { Editor } from '../editor.svelte';
	import { toScreen } from '../camera';
	import { youtubeEmbed } from '../youtube';

	let { editor }: { editor: Editor } = $props();

	const shape = $derived.by(() => {
		void editor.version;
		return editor.playing ? (editor.scene.get(editor.playing) ?? null) : null;
	});

	const src = $derived(shape?.url ? youtubeEmbed(shape.url) : null);

	const view = $derived.by(() => {
		if (!shape) return null;
		const c = editor.camera;
		const at = toScreen(c, { x: shape.x, y: shape.y });
		return { x: at.x, y: at.y, w: shape.w * c.z, h: shape.h * c.z, rot: shape.rot };
	});

	function close() {
		editor.playing = null;
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && editor.playing && close()} />

{#if view && src}
	<div
		class="pointer-events-auto absolute z-30"
		style:left="{view.x}px"
		style:top="{view.y}px"
		style:width="{view.w}px"
		style:height="{view.h}px"
		style:transform="rotate({view.rot}rad)"
		style:transform-origin="center"
		data-board-ui
		data-testid="video-player"
	>
		<iframe
			class="h-full w-full rounded-fc-md bg-fc-component shadow-lg"
			{src}
			title={shape?.title || 'Vidéo YouTube'}
			allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
			allowfullscreen
			referrerpolicy="strict-origin-when-cross-origin"
		></iframe>
		<IconButton
			variant="secondary"
			class="absolute -top-11 right-0 shadow-lg"
			aria-label="Fermer la vidéo"
			onclick={close}
		>
			<Icon icon={icons.close} size={16} />
		</IconButton>
	</div>
{/if}
