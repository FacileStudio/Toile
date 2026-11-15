<script lang="ts">
	import { Button, Icon, SettingsRow, SettingsSection, icons } from '@facile/muse';
	import { SUITE } from '$lib/components/landing/content';
	import { t } from '$lib/i18n/index.svelte';
</script>

<!--
@component
The suite apps Toile reaches on the person's behalf, and where to review that access on Facile.
-->

<SettingsSection
	title={t('settings.appsTitle')}
	description={t('settings.appsBody')}
>
	{#snippet actions()}
		<Button href="https://sso.facile.studio/account/apps" variant="secondary" size="sm" iconRight={icons.external}
			>{t('settings.manageAccess')}</Button
		>
	{/snippet}
	{#each SUITE as app (app.title)}
		<SettingsRow label={app.title} description={t(app.body)}>
			<Button
				href={app.href}
				target="_blank"
				rel="noopener noreferrer"
				variant="ghost"
				size="sm"
				aria-label={t('settings.openApp', { app: app.title })}
			>
				<Icon icon={icons.external} size={16} />
			</Button>
		</SettingsRow>
	{/each}
</SettingsSection>
