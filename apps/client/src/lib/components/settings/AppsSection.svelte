<script lang="ts">
	import { Button, Icon, SettingsRow, SettingsSection, icons } from '@facile/muse';
	import { SUITE } from '$lib/components/landing/content';
</script>

<!--
@component
The suite apps Toile reaches on the person's behalf, and where to review that access on Facile.
-->

<SettingsSection
	title="Applications connectées"
	description="Toile agit dans ces applications avec votre compte Facile, uniquement quand vous le demandez sur un tableau."
>
	{#snippet actions()}
		<Button href="https://sso.facile.studio/account/apps" variant="secondary" size="sm" iconRight={icons.external}
			>Gérer les accès</Button
		>
	{/snippet}
	{#each SUITE as app (app.title)}
		<SettingsRow label={app.title} description={app.body}>
			<Button
				href={app.href}
				target="_blank"
				rel="noopener noreferrer"
				variant="ghost"
				size="sm"
				aria-label="Ouvrir {app.title}"
			>
				<Icon icon={icons.external} size={16} />
			</Button>
		</SettingsRow>
	{/each}
</SettingsSection>
