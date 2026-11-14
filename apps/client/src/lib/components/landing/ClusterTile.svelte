<script lang="ts">
	import { Blob, thinking } from '@facile/blob';
	import { Button, USER_COLORS } from '@facile/muse';
	import { Bar, Stage, Tile, type Act } from '@facile/mise-en-scene';
	import Sticky from './Sticky.svelte';

	const NOTES = [
		{ box: { x: 24, y: 54, w: 50, h: 42 }, from: [96, 2], color: USER_COLORS[0] },
		{ box: { x: 82, y: 54, w: 50, h: 42 }, from: [118, 36], color: USER_COLORS[0] },
		{ box: { x: 156, y: 54, w: 50, h: 42 }, from: [-118, 44], color: USER_COLORS[4] },
		{ box: { x: 214, y: 54, w: 50, h: 42 }, from: [-38, -14], color: USER_COLORS[4] }
	];
	const THEMES = ['Accueil', 'Paiement'];

	let sorted = $state(false);
	let busy = $state(false);

	function reset() {
		sorted = false;
		busy = false;
	}

	function poster() {
		sorted = true;
	}

	async function script(act: Act) {
		await act.wait(1200);
		busy = true;
		await act.wait(1300);
		busy = false;
		sorted = true;
		await act.wait(3200);
		sorted = false;
		await act.wait(1200);
	}
</script>

<Tile title="Bulle range vos idées" body="Sélectionnez des post-it, l’IA les regroupe par thème dans des cadres.">
	<Stage width={320} height={240} {script} {reset} {poster}>
		{#snippet children()}
			<div class="board absolute top-5 right-5 bottom-5 left-5 overflow-hidden rounded-fc-lg bg-fc-page">
				{#each THEMES as theme, at (theme)}
					<div class="frame absolute top-[38px] h-[72px] w-[124px] rounded-fc-lg bg-fc-surface" class:shown={sorted} style:left="{14 + at * 132}px">
						<span class="absolute -top-5 left-1 text-fc-xs font-medium text-fc-fg-muted">{theme}</span>
					</div>
				{/each}
				{#each NOTES as note, at (at)}
					<div class="move absolute inset-0" style:translate={sorted ? '0 0' : `${note.from[0]}px ${note.from[1]}px`}>
						<Sticky box={note.box} color={note.color}><Bar w="75%" /><Bar w="50%" /></Sticky>
					</div>
				{/each}
				<div class="absolute bottom-3 left-1/2 flex -translate-x-1/2 items-center gap-2 rounded-fc-pill bg-fc-component p-1.5 pl-2 shadow-lg">
					<Blob name="Bulle" size={24} animate={busy ? 'always' : undefined} expression={busy ? thinking : undefined} />
					<Button variant="secondary" size="sm" class="whitespace-nowrap {busy ? 'opacity-70' : ''}">Ranger 4 post-it</Button>
				</div>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.board {
		background-image: radial-gradient(color-mix(in oklch, var(--color-fc-fg) 16%, transparent) 1px, transparent 1px);
		background-size: 18px 18px;
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.28);
	}
	.move {
		transition: translate 0.8s cubic-bezier(0.45, 0, 0.1, 1);
	}
	.frame {
		opacity: 0;
		scale: 0.94;
		transition:
			opacity 0.35s ease,
			scale 0.45s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.frame.shown {
		opacity: 1;
		scale: 1;
		transition-delay: 0.45s;
	}
</style>
