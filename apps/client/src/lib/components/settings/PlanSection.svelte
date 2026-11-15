<script lang="ts">
	import { Button, PricingPlans, SettingsSection, icons } from '@facile/muse';
	import { t } from '$lib/i18n/index.svelte';

	const SPACES = 'https://sso.facile.studio/spaces';
</script>

<!--
@component
The plan: what is free, what a paid plan adds, and the suite catalog to upgrade from. Billing
itself lives on Facile, per space.
-->

<div class="flex flex-col gap-8">
	<SettingsSection
		title={t('settings.plan')}
		description={t('settings.planBody')}
	>
		{#snippet actions()}
			<Button href={SPACES} variant="secondary" size="sm" iconRight={icons.external}>{t('settings.manageSubscription')}</Button>
		{/snippet}
	</SettingsSection>
	<PricingPlans
		title={t('settings.plans')}
		headingLevel={2}
		highlight={['ai.access', 'toile.upload_mb']}
		ctaHref={SPACES}
		synthesizeFree
	/>
</div>
