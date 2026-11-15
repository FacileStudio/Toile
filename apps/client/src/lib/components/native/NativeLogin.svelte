<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
	import { Alert, Button, Field, Icon, Input, SignInWithFacile } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import { cleanCode, closeLoginBrowser, redeem, startNativeLogin } from '$lib/native';

	let { onsignedin }: { onsignedin: () => Promise<void> } = $props();

	let busy = $state(false);
	let error = $state('');
	let awaitingCode = $state(false);
	let code = $state('');

	function failure(cause: unknown): string {
		return cause instanceof Error ? cause.message : t('native.failed');
	}

	async function start(event: MouseEvent) {
		event.preventDefault();
		if (busy) return;
		busy = true;
		error = '';
		try {
			if (await startNativeLogin()) await onsignedin();
			else awaitingCode = true;
		} catch (cause) {
			error = failure(cause);
		} finally {
			busy = false;
		}
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const clean = cleanCode(code);
		if (!clean) {
			error = t('native.codeMissing');
			return;
		}
		busy = true;
		error = '';
		try {
			await redeem(clean);
			await closeLoginBrowser();
			await onsignedin();
		} catch (cause) {
			error = failure(cause);
		} finally {
			busy = false;
		}
	}
</script>

<div class="fixed inset-0 flex flex-col overflow-hidden bg-fc-page text-fc-fg">
	<header class="flex h-14 items-center gap-2 px-5 text-fc-md font-semibold">
		<Icon icon={logoIcon} size={22} />Toile
	</header>

	<main class="flex min-h-0 flex-1 flex-col items-center justify-center gap-8 overflow-y-auto px-8 text-center">
		<div class="flex size-32 items-center justify-center rounded-[2rem] bg-fc-surface shadow-sm">
			<Icon icon={logoIcon} size={56} />
		</div>
		<div class="flex max-w-xs flex-col gap-3">
			<h1 class="text-fc-3xl leading-tight font-semibold tracking-tight">{t('native.signIn')}</h1>
			<p class="text-fc-md leading-relaxed text-fc-fg-muted">
				{awaitingCode ? t('native.pasteCode') : t('native.account')}
			</p>
		</div>
		{#if error}<Alert tone="danger" class="w-full max-w-sm text-left">{error}</Alert>{/if}
	</main>

	<footer class="flex flex-col items-center gap-3 px-6 pt-4 pb-6">
		{#if awaitingCode}
			<form class="flex w-full max-w-sm flex-col gap-3" onsubmit={submit}>
				<Field label={t('native.code')}>
					<Input bind:value={code} autocomplete="one-time-code" autocapitalize="off" spellcheck={false} />
				</Field>
				<Button type="submit" size="lg" disabled={busy}>{t('native.submit')}</Button>
				<Button variant="ghost" size="sm" disabled={busy} onclick={start}>{t('native.reopen')}</Button>
			</form>
		{:else}
			<SignInWithFacile size="lg" fullWidth class="max-w-sm" loading={busy} onclick={start} />
		{/if}
	</footer>
</div>
