<script lang="ts">
	import { Icon, USER_COLORS, icons } from '@facile/muse';
	import { Bar, Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n/index.svelte';
	import { suiteIcons } from '$lib/suiteIcons';
	import Sticky from './Sticky.svelte';

	let pasting = $state(false);
	let carded = $state(false);

	function reset() {
		pasting = false;
		carded = false;
	}

	function poster() {
		carded = true;
	}

	async function script(act: Act) {
		await act.wait(900);
		pasting = true;
		await act.wait(1100);
		pasting = false;
		carded = true;
		await act.wait(3400);
		carded = false;
		await act.wait(900);
	}
</script>

<Tile title={t('opus.title')} body={t('opus.body')}>
	<Stage width={320} height={240} {script} {reset} {poster}>
		{#snippet children()}
			<div class="board absolute top-5 right-5 bottom-5 left-5 overflow-hidden rounded-fc-lg bg-fc-page">
				<Sticky box={{ x: 14, y: 52, w: 104, h: 96 }} color={USER_COLORS[3]}>
					{t('opus.sticky')}<Bar w="80%" /><Bar w="55%" />
				</Sticky>
				<span
					class="keys absolute top-[86px] left-[170px] flex h-8 items-center gap-1 rounded-fc-pill bg-fc-component px-3 font-fc-mono text-fc-xs text-fc-fg shadow-lg"
					class:shown={pasting}
				>
					Ctrl V
				</span>
				<div
					class="card absolute top-[52px] left-[134px] flex h-[96px] w-[130px] flex-col justify-between rounded-fc-lg bg-fc-component p-3 shadow-lg"
					class:shown={carded}
				>
					<span class="flex items-center justify-between text-fc-fg-muted">
						<span class="flex size-6 items-center justify-center rounded-fc-md bg-fc-surface text-fc-fg">
							<Icon icon={suiteIcons.opus} size={16} />
						</span>
						<Icon icon={icons.external} size={14} />
					</span>
					<span class="line-clamp-2 text-fc-xs leading-4 font-semibold text-fc-fg">{t('opus.task')}</span>
					<span class="truncate text-[10px] leading-3 text-fc-fg-muted">opus.facile.studio</span>
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
	.keys,
	.card {
		opacity: 0;
		scale: 0.85;
		transition:
			opacity 0.25s ease,
			scale 0.4s cubic-bezier(0.34, 1.4, 0.64, 1);
	}
	.shown {
		opacity: 1;
		scale: 1;
	}
</style>
