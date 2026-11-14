<script lang="ts">
	import { Icon, USER_COLORS } from '@facile/muse';
	import { Bar, Stage, Tile, grow, type Act } from '@facile/mise-en-scene';
	import { toolIcons } from '$lib/canvas/icons';
	import Sticky from './Sticky.svelte';

	const QUESTION = 'On garde ce titre ?';
	const ANSWER = 'Oui, validé.';

	let pinned = $state(false);
	let asked = $state('');
	let replied = $state(false);

	function reset() {
		pinned = false;
		asked = '';
		replied = false;
	}

	function poster() {
		pinned = true;
		asked = QUESTION;
		replied = true;
	}

	async function script(act: Act) {
		await act.wait(700);
		pinned = true;
		await act.wait(450);
		await act.type(QUESTION, (value) => (asked = value), 55);
		await act.wait(1100);
		replied = true;
		await act.wait(2600);
		reset();
		await act.wait(600);
	}
</script>

<Tile title="Commentez sur place" body="Un fil de discussion posé sur le tableau, là où se trouve l’idée.">
	<Stage width={320} height={240} {script} {reset} {poster}>
		{#snippet children()}
			<div class="board absolute top-5 right-5 bottom-5 left-5 overflow-hidden rounded-fc-lg bg-fc-page">
				<Sticky box={{ x: 14, y: 48, w: 104, h: 88 }} color={USER_COLORS[0]}>
					Nouveau logo<Bar w="80%" /><Bar w="55%" />
				</Sticky>
				<Sticky box={{ x: 30, y: 150, w: 88, h: 36 }} color={USER_COLORS[4]}><Bar w="70%" /></Sticky>
				<span
					class="pin absolute top-[48px] left-[104px] flex h-9 min-w-9 items-center gap-1 rounded-fc-pill rounded-bl-fc-xs bg-fc-accent px-2 text-fc-xs font-medium text-fc-accent-fg shadow-lg"
					class:shown={pinned}
				>
					<Icon icon={toolIcons.comment} size={16} />{replied ? 2 : 1}
				</span>
				<div
					class="panel absolute top-14 right-3 flex w-[136px] flex-col gap-2 rounded-fc-lg bg-fc-component p-3 shadow-lg"
					class:shown={pinned}
				>
					<span class="text-fc-xs font-semibold text-fc-fg">Commentaires</span>
					<div class="flex flex-col gap-0.5 rounded-fc-md bg-fc-surface p-2">
						<span class="text-[10px] leading-3 font-medium text-fc-fg">Léa</span>
						<p class="min-h-4 text-fc-xs leading-4 text-fc-fg">{asked}</p>
					</div>
					{#if replied}
						<div class="flex flex-col gap-0.5 pl-3" transition:grow={{ duration: 300 }}>
							<span class="text-[10px] leading-3 font-medium text-fc-fg">Malik</span>
							<p class="text-fc-xs leading-4 text-fc-fg">{ANSWER}</p>
						</div>
					{:else}
						<div class="pl-3"><Bar w="60%" /></div>
					{/if}
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
	.pin {
		translate: 0 -100%;
		opacity: 0;
		scale: 0.6;
		transform-origin: bottom left;
		transition:
			opacity 0.25s ease,
			scale 0.4s cubic-bezier(0.34, 1.4, 0.64, 1);
	}
	.panel {
		opacity: 0;
		translate: 0 6px;
		transition:
			opacity 0.3s ease,
			translate 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.shown {
		opacity: 1;
		scale: 1;
	}
	.panel.shown {
		translate: 0 0;
	}
</style>
