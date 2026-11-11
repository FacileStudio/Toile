<script lang="ts">
	import { Button, PricingPlans, SettingsSection, icons } from '@facile/muse';

	const SPACES = 'https://sso.facile.studio/spaces';
</script>

<!--
@component
The plan: what is free, what a paid plan adds, and the suite catalog to upgrade from. Billing
itself lives on Facile, per space.
-->

<div class="flex flex-col gap-8">
	<SettingsSection
		title="Forfait"
		description="Toile est gratuit, sans limite de tableaux. Un forfait payant ajoute l’IA avec Bulle et des imports plus lourds."
	>
		{#snippet actions()}
			<Button href={SPACES} variant="secondary" size="sm" iconRight={icons.external}>Gérer l’abonnement</Button>
		{/snippet}
	</SettingsSection>
	<PricingPlans
		title="Les forfaits Facile"
		headingLevel={2}
		highlight={['ai.access', 'toile.upload_mb']}
		ctaHref={SPACES}
		synthesizeFree
	/>
</div>
