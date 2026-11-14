<script lang="ts">
	import {
		FeatureSplit,
		LANDING_HEADING,
		LandingCta,
		LandingFooter,
		LandingHeader,
		PricingPlans,
		SignInWithFacile,
		TextElevate,
		icons,
		type FeatureItem,
		type WorksWithApp
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon } from '$lib/brand';
	import ClusterTile from '$lib/components/landing/ClusterTile.svelte';
	import CommentTile from '$lib/components/landing/CommentTile.svelte';
	import HeroBoard from '$lib/components/landing/HeroBoard.svelte';
	import OpusTile from '$lib/components/landing/OpusTile.svelte';
	import WorksWithRow from '$lib/components/landing/WorksWithRow.svelte';
	import { rememberLocale, t } from '$lib/i18n/index.svelte';
	import { suiteIcons } from '$lib/suiteIcons';

	const WIDTH = 'mx-auto w-full max-w-fc-xl px-4 sm:px-6';

	const links = $derived([
		{ href: '#features', label: t('landing.nav.features') },
		{ href: '#pricing', label: t('landing.nav.pricing') }
	]);

	const features = $derived<FeatureItem[]>([
		{ icon: icons.compass, title: t('landing.feature.canvas.title'), text: t('landing.feature.canvas.text') },
		{ icon: icons.usersGroup, title: t('landing.feature.live.title'), text: t('landing.feature.live.text') },
		{ icon: icons.globe, title: t('landing.feature.guests.title'), text: t('landing.feature.guests.text') },
		{ icon: icons.download, title: t('landing.feature.export.title'), text: t('landing.feature.export.text') }
	]);

	const apps = $derived<WorksWithApp[]>([
		{ name: 'Opus', icon: suiteIcons.opus, line: t('with.opus'), href: 'https://opus.facile.studio' },
		{ name: 'Docs', icon: suiteIcons.docs, line: t('with.docs'), href: 'https://docs.facile.studio' },
		{ name: 'Nuage', icon: suiteIcons.nuage, line: t('with.nuage'), href: 'https://nuage.facile.studio' },
		{ name: 'Bulle', icon: suiteIcons.bulle, line: t('with.bulle'), href: 'https://bulle.facile.studio' }
	]);

	const columns = $derived([
		{ title: 'Toile', links },
		{
			title: 'Facile',
			links: [
				...apps.map((app) => ({ href: app.href ?? '', label: app.name, external: true })),
				{ href: 'https://suite.facile.studio', label: 'Suite', external: true }
			]
		}
	]);

	$effect(() => {
		document.querySelector('meta[name="description"]')?.setAttribute('content', t('landing.meta'));
	});
</script>

<svelte:head>
	<title>{t('landing.title')}</title>
</svelte:head>

<div class="min-h-dvh bg-fc-page text-fc-fg">
	<LandingHeader appName="Toile" icon={logoIcon} {links} skipHref="#main">
		{#snippet actions()}
			<SignInWithFacile href={backend.loginUrl()} size="sm" />
		{/snippet}
	</LandingHeader>

	<main id="main" class="flex flex-col gap-24 pb-24">
		<section class="{WIDTH} flex flex-col items-center gap-10 pt-12 md:pt-20">
			<div class="flex max-w-fc-md flex-col items-center gap-6 text-center">
				<h1 class="font-fc-title text-fc-3xl font-bold text-balance">
					{#key t('landing.slogan')}<TextElevate text={t('landing.slogan')} delay={0.1} />{/key}
				</h1>
				<p class="text-fc-md text-balance text-fc-fg-muted">{t('landing.subtitle')}</p>
				<LandingCta signInHref={backend.loginUrl()} align="center" />
			</div>
			<HeroBoard />
		</section>

		<section aria-labelledby="demos-title" class={WIDTH}>
			<h2 id="demos-title" class="mb-10 text-center {LANDING_HEADING}">{t('landing.demos.title')}</h2>
			<div class="grid gap-6 md:grid-cols-3">
				<CommentTile />
				<ClusterTile />
				<OpusTile />
			</div>
		</section>

		<WorksWithRow class={WIDTH} {apps} description={t('with.line')} />

		<FeatureSplit
			id="features"
			class="{WIDTH} scroll-mt-20"
			title={t('landing.features.title')}
			description={t('landing.features.line')}
			{features}
		/>

		<PricingPlans
			id="pricing"
			class="{WIDTH} scroll-mt-20"
			title={t('landing.pricing.title')}
			headingClass={LANDING_HEADING}
			highlight={['ai.access']}
			highlightOnly
			notes={{ solo: t('landing.plan.free'), '*': t('landing.plan.paid') }}
			recommended="pro"
			suiteHref="https://suite.facile.studio/#tarifs"
		/>

		<section class="mx-auto flex w-full max-w-fc-md flex-col items-center gap-6 px-4 text-center sm:px-6">
			<h2 class={LANDING_HEADING}>{t('landing.cta.title')}</h2>
			<LandingCta signInHref={backend.loginUrl()} align="center" />
		</section>
	</main>

	<LandingFooter
		appName="Toile"
		icon={logoIcon}
		tagline={t('landing.tagline')}
		{columns}
		legal={{ onLocaleChange: rememberLocale }}
	/>
</div>
