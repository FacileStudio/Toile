<script lang="ts">
	import { Icon, LANDING_HEADING, getI18n, worksWithApps, type WorksWithApp } from '@facile/muse';

	let { apps, description, class: className = '' }: { apps: WorksWithApp[]; description?: string; class?: string } = $props();

	const i18n = getI18n();
	const uid = $props.id();
	const items = $derived(worksWithApps(apps));
</script>

<!--
@component
The suite apps Toile connects to, as one compact row: icon, name and one line each, no cards, so it never reads as a second card grid next to the demo tiles.
-->

{#if items.length}
	<section aria-labelledby="{uid}-title" class="flex min-w-0 flex-col gap-6 {className}">
		<div class="flex flex-col gap-3">
			<h2 id="{uid}-title" class={LANDING_HEADING}>{i18n.t('landing.worksWith')}</h2>
			{#if description}<p class="text-fc-md text-fc-fg-muted">{description}</p>{/if}
		</div>
		<ul class="grid min-w-0 gap-x-8 gap-y-5 sm:grid-cols-2 lg:grid-cols-4">
			{#each items as app (app.name)}
				<li class="flex min-w-0 items-start gap-3">
					<Icon icon={app.icon} size={22} class="mt-0.5 shrink-0 text-fc-fg" />
					<p class="min-w-0 text-fc-sm text-fc-fg-muted">
						{#if app.href}
							<a
								href={app.href}
								class="font-semibold text-fc-fg underline-offset-4 hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
								>{app.name}</a
							>
						{:else}
							<span class="font-semibold text-fc-fg">{app.name}</span>
						{/if}
						{app.line}
					</p>
				</li>
			{/each}
		</ul>
	</section>
{/if}
