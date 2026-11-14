<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		AuthScreen,
		ConfirmModal,
		Icon,
		MobileNav,
		SideBar,
		Spinner,
		Toaster,
		Topbar,
		provideLocale,
		type UserMenuOptions
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { boardIcon, logoIcon } from '$lib/brand';
	import { reconnect } from '$lib/reconnect.svelte';
	import { takeReturn } from '$lib/returnTo';
	import NativeGate from '$lib/components/native/NativeGate.svelte';
	import { isApp } from '$lib/platform';
	import { session } from '$lib/session.svelte';
	import { theme } from '$lib/theme';

	let { children } = $props();

	let collapsed = $state(typeof window !== 'undefined' && window.innerWidth < 1280);

	const path = $derived(page.url.pathname);
	const onSettings = $derived(path.startsWith('/settings'));
	const landing = $derived(path === '/');
	const fullscreen = $derived(path.startsWith('/s/') || (path.startsWith('/b/') && !!session.user));

	provideLocale(() => 'fr');
	theme.restore();

	$effect(() => {
		if (!session.ready) void session.load();
	});

	$effect(() => {
		if (session.ready && session.user && landing) void goto(takeReturn() ?? '/boards', { replaceState: true });
	});

	const pages = $derived([
		{ label: 'Tableaux', href: '/boards', icon: boardIcon, active: path.startsWith('/boards') }
	]);
	const navUser = $derived({
		name: session.user?.name || session.user?.email || 'Compte',
		email: session.user?.email,
		avatar: session.user?.avatar_url || undefined
	});
	const userMenu = $derived<UserMenuOptions>({
		settingsHref: '/settings',
		showLanguage: false,
		onThemeChange: (mode) => theme.set(mode),
		onSignOut: async () => {
			await session.logout();
			await goto('/');
		}
	});
	const spaces = $derived(session.spaces.map((space) => ({ id: space.id, name: space.name })));
</script>

{#if !session.ready}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-page"><Spinner /></div>
{:else if fullscreen}
	{@render children()}
{:else if !session.user && isApp()}
	<NativeGate onsignedin={() => session.load()} />
{:else if !session.user && landing}
	{@render children()}
{:else if !session.user}
	<AuthScreen appName="Toile" href={backend.loginUrl()} />
{:else}
	<div class="flex h-dvh w-full overflow-hidden bg-fc-page">
		<div class="hidden h-full shrink-0 p-3 md:flex">
			<SideBar
				icon={logoIcon}
				title="Toile"
				bind:collapsed
				{pages}
				{spaces}
				activeSpaceId={session.spaceId}
				onSpaceSelect={(id) => session.select(id)}
				personalSpaceLabel="Personnel"
				user={navUser}
				userHref="/settings"
				userActive={onSettings}
				{userMenu}
				class="h-full"
			/>
		</div>

		<main class="min-w-0 flex-1 overflow-auto overscroll-contain pb-28 md:pb-0">
			<Topbar class="md:hidden">
				<span class="flex flex-1 items-center gap-2 text-fc-md font-semibold text-fc-fg">
					<Icon icon={logoIcon} size={20} />Toile
				</span>
			</Topbar>
			{@render children()}
		</main>

		<MobileNav items={pages} user={navUser} profileHref="/settings" profileActive={onSettings} {userMenu} />
	</div>
{/if}

<ConfirmModal
	bind:open={reconnect.needed}
	title="Reconnectez votre session Facile"
	description="Votre session Facile a expiré. Reconnectez-vous pour utiliser les autres apps de la suite depuis Toile."
	confirmLabel="Se reconnecter"
	cancelLabel="Plus tard"
	onConfirm={() => window.location.assign(backend.loginUrl())}
/>

<Toaster />
