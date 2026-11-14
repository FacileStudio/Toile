<script lang="ts">
	import { Icon, icons } from '@facile/muse';
	import { Roll, Stage, Tile, grow, type Act } from '@facile/mise-en-scene';

	const ROLES = ['Lecture seule', 'Commentaire', 'Modification'];

	let role = $state(0);
	let pressed = $state(false);
	let linked = $state(false);
	let copied = $state(false);

	function reset() {
		role = 0;
		pressed = false;
		linked = false;
		copied = false;
	}

	function poster() {
		role = 1;
		linked = true;
		copied = true;
	}

	async function script(act: Act) {
		await act.wait(1000);
		role = 1;
		await act.wait(900);
		pressed = true;
		await act.wait(180);
		pressed = false;
		linked = true;
		await act.wait(700);
		copied = true;
		await act.wait(2800);
		reset();
		await act.wait(600);
	}
</script>

<Tile title="Invitez sans compte" body="Un lien en lecture, commentaire ou modification, révocable à tout moment.">
	<Stage width={320} height={240} {script} {reset} {poster}>
		{#snippet children()}
			<div class="panel absolute top-5 right-5 left-5 flex flex-col gap-3 rounded-fc-lg bg-fc-page p-4">
				<div class="flex items-center justify-between">
					<span class="text-fc-sm font-semibold text-fc-fg">Partager le tableau</span>
					<Icon icon={icons.close} size={16} class="text-fc-fg-muted" />
				</div>
				<div class="flex items-center gap-2">
					<span class="flex h-9 min-w-0 flex-1 items-center justify-between gap-1 rounded-fc-md bg-fc-field px-3 text-fc-xs text-fc-fg">
						<Roll value={ROLES[role]} class="h-4 w-24 leading-4" />
						<Icon icon={icons.chevronDown} size={14} class="text-fc-fg-muted" />
					</span>
					<span
						class="press flex h-9 shrink-0 items-center gap-1 rounded-fc-pill bg-fc-accent px-3 text-fc-xs font-medium text-fc-accent-fg"
						class:pressed
					>
						<Icon icon={icons.plus} size={14} />Nouveau lien
					</span>
				</div>
				{#if linked}
					<div transition:grow={{ duration: 320 }}>
						<div class="flex items-center gap-2 rounded-fc-md bg-fc-surface py-1.5 pr-2 pl-3">
							<span class="min-w-0 flex-1 truncate font-fc-mono text-[11px] text-fc-fg">toile.facile.studio/s/k3f9q2</span>
							<span class="flex size-6 items-center justify-center {copied ? 'text-fc-success' : 'text-fc-fg-muted'}">
								<Icon icon={copied ? icons.check : icons.copy} size={14} />
							</span>
						</div>
						<p class="pt-2 text-[11px] leading-4 text-fc-fg-muted">{copied ? 'Lien copié.' : ' '}</p>
					</div>
				{/if}
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.panel {
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.28);
	}
	.press {
		transition: scale 0.15s ease;
	}
	.pressed {
		scale: 0.92;
	}
</style>
