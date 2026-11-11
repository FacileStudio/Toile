<script lang="ts">
	import { Icon, LandingHeader, SignInWithFacile } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon } from '$lib/brand';
</script>

<svelte:head><title>Toile · le tableau blanc de Facile</title></svelte:head>

<LandingHeader appName="Toile" icon={logoIcon} skipHref="#main">
	{#snippet actions()}
		<SignInWithFacile href={backend.loginUrl()} size="sm" />
	{/snippet}
</LandingHeader>

<main id="main" class="mx-auto flex min-h-[70dvh] w-full max-w-3xl flex-col items-center justify-center gap-6 px-4 py-16 text-center">
	<Icon icon={logoIcon} size={56} class="text-fc-fg" />
	<h1 class="text-fc-3xl font-semibold text-fc-fg">Toile</h1>
	<p class="max-w-xl text-fc-md text-fc-fg-muted">
		Le tableau blanc infini de Facile : post-it, formes, flèches et dessin à main levée, à plusieurs et en direct.
	</p>
	<SignInWithFacile href={backend.loginUrl()} size="lg" />
</main>
