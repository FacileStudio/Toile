<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { PreferencesSection, ProfileSection, SettingsLayout, icons, sectionHref, settingsRoute } from '@facile/muse';
	import AppsSection from '$lib/components/settings/AppsSection.svelte';
	import PlanSection from '$lib/components/settings/PlanSection.svelte';
	import ShortcutsSection from '$lib/components/settings/ShortcutsSection.svelte';
	import { session } from '$lib/session.svelte';
	import { theme } from '$lib/theme';
	import { rememberLocale, t } from '$lib/i18n/index.svelte';

	const sections = $derived(
		[
			{ id: 'profile', label: t('settings.profile'), icon: icons.userCircle },
			{ id: 'preferences', label: t('settings.appearance'), icon: icons.palette },
			{ id: 'shortcuts', label: t('settings.shortcuts'), icon: icons.bolt },
			{ id: 'apps', label: t('settings.apps'), icon: icons.plug },
			{ id: 'subscription', label: t('settings.plan'), icon: icons.card }
		].map((section) => ({ ...section, href: sectionHref('/settings', section.id) }))
	);

	const route = $derived(settingsRoute(page.params.section, sections));
	const role = $derived(
		session.space ? { member: t('settings.member'), admin: t('settings.admin'), owner: t('settings.owner') }[session.space.role] : undefined
	);

	$effect(() => {
		if (!route.known) void goto('/settings', { replaceState: true });
	});
</script>

<svelte:head><title>{t('settings.title')} · Toile</title></svelte:head>

<div class="mx-auto w-full max-w-5xl px-4 py-6 sm:px-6 md:py-10">
	<SettingsLayout {sections} active={page.params.section} backHref="/settings" title={t('settings.title')}>
		{#snippet children(current)}
			{#if current === 'profile'}
				<ProfileSection
					title={t('settings.profile')}
					name={session.user?.name || session.user?.email || ''}
					email={session.user?.email}
					avatar={session.user?.avatar_url}
					{role}
				/>
			{:else if current === 'preferences'}
				<PreferencesSection
					title={t('settings.appearance')}
					onLocaleChange={rememberLocale}
					theme={theme.mode}
					onThemeChange={(mode) => theme.set(mode)}
				/>
			{:else if current === 'shortcuts'}
				<ShortcutsSection />
			{:else if current === 'apps'}
				<AppsSection />
			{:else}
				<PlanSection />
			{/if}
		{/snippet}
	</SettingsLayout>
</div>
