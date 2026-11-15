<script lang="ts">
	import { page } from '$app/state';
	import { Button, EmptyState, icons } from '@facile/muse';
	import { t } from '$lib/i18n/index.svelte';

	const missing = $derived(page.status === 404);
</script>

<svelte:head><title>{missing ? t('error.missing') : t('error.title')} · Toile</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col px-4 py-16 sm:px-6">
	<EmptyState
		icon={missing ? icons.search : icons.warning}
		title={missing ? t('error.missing') : t('error.failed')}
		description={missing ? t('error.missingBody') : t('error.failedBody')}
	>
		<Button href="/" icon={icons.home}>{t('common.home')}</Button>
	</EmptyState>
</div>
