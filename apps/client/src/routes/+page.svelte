<script lang="ts">
	import { Button, LandingFooter, LandingHeader, SignInWithFacile, TextElevate, icons } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon } from '$lib/brand';
	import FeatureCard from '$lib/components/landing/FeatureCard.svelte';
	import HeroBoard from '$lib/components/landing/HeroBoard.svelte';
	import { FEATURES, SUITE } from '$lib/components/landing/content';

	const links = [{ href: '#features', label: 'Fonctionnalités' }];

	const columns = [
		{ title: 'Toile', links },
		{
			title: 'Facile',
			links: [
				...SUITE.map((app) => ({ href: app.href, label: app.title, external: true })),
				{ href: 'https://sso.facile.studio/signup', label: 'Créer un compte', external: true }
			]
		}
	];
</script>

<svelte:head>
	<title>Toile · Le tableau blanc infini de Facile</title>
</svelte:head>

<div class="min-h-dvh bg-fc-page text-fc-fg">
	<LandingHeader appName="Toile" icon={logoIcon} {links} skipHref="#main">
		{#snippet actions()}
			<SignInWithFacile href={backend.loginUrl()} size="sm" />
		{/snippet}
	</LandingHeader>

	<main id="main" class="flex flex-col gap-24 pb-24">
		<section class="mx-auto flex w-full max-w-fc-xl flex-col items-center gap-10 px-4 pt-12 sm:px-6 md:pt-20">
			<div class="flex max-w-fc-md flex-col items-center gap-6 text-center">
				<h1 class="font-fc-title text-fc-3xl font-bold text-balance">
					<TextElevate text="Le tableau blanc infini de Facile" delay={0.1} />
				</h1>
				<p class="text-fc-md text-balance text-fc-fg-muted">Post-it, formes et flèches, à plusieurs et en direct.</p>
				<div class="flex flex-wrap items-center justify-center gap-3">
					<SignInWithFacile href={backend.loginUrl()} size="lg" />
					<Button href="#features" variant="secondary" size="lg" icon={icons.chevronDown}>Découvrir</Button>
				</div>
			</div>
			<HeroBoard />
		</section>

		<section id="features" aria-labelledby="features-title" class="mx-auto flex w-full max-w-fc-xl flex-col gap-8 px-4 sm:px-6">
			<h2 id="features-title" class="font-fc-title text-fc-2xl font-bold">Réfléchir à plusieurs</h2>
			<ul class="grid gap-4 md:grid-cols-2">
				{#each FEATURES as feature (feature.title)}
					<li><FeatureCard {feature} /></li>
				{/each}
			</ul>
		</section>

		<section class="mx-auto flex w-full max-w-fc-md flex-col items-center gap-6 px-4 text-center sm:px-6">
			<h2 class="font-fc-title text-fc-2xl font-bold">Votre prochain atelier commence ici</h2>
			<SignInWithFacile href={backend.loginUrl()} size="lg" />
		</section>
	</main>

	<LandingFooter
		appName="Toile"
		icon={logoIcon}
		tagline="Le tableau blanc infini de Facile."
		{columns}
		legal={{ hideLocaleSwitch: true }}
	/>
</div>
