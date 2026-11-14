<script lang="ts">
	import { page } from '$app/state';
	import { Alert, ApprovalCard, Button, EmptyState, PageHeader, Spinner, StatusDot, icons, toast } from '@facile/muse';
	import { STATES, notOwnSession, pending, targetBoard, type PendingAction } from '$lib/api/pending';
	import { backend } from '$lib/backend';
	import { formatDate } from '$lib/format';
	import { isApp } from '$lib/platform';
	import { rememberReturn } from '$lib/returnTo';

	const id = $derived(page.url.searchParams.get('action') ?? '');

	let action = $state<PendingAction | null | undefined>(undefined);
	let outcome = $state<'confirmed' | 'dismissed' | null>(null);
	let busy = $state(false);
	let refused = $state(false);

	function failed(error: unknown, fallback: string) {
		refused = notOwnSession(error);
		if (!refused) toast.danger(error instanceof Error ? error.message : fallback);
	}

	function signIn() {
		rememberReturn(page.url.pathname + page.url.search);
		window.location.assign(backend.loginUrl());
	}

	$effect(() => {
		const wanted = id;
		action = undefined;
		outcome = null;
		refused = false;
		pending
			.list()
			.then((list) => (action = list.find((item) => item.id === wanted) ?? null))
			.catch((error) => {
				action = null;
				failed(error, 'Demandes indisponibles');
			});
	});

	const boardId = $derived(action ? targetBoard(action) : null);
	const status = $derived(action ? (STATES[action.state] ?? STATES.missing) : null);
	const approval = $derived(
		action ? { id: action.id, tool: action.kind, summary: action.summary, args: parsed(action.target) } : null
	);

	function parsed(target: string): Record<string, unknown> | string {
		try {
			return JSON.parse(target) as Record<string, unknown>;
		} catch {
			return target;
		}
	}

	async function answer(confirm: boolean) {
		if (!action) return;
		busy = true;
		try {
			if (confirm) await pending.confirm(action.id);
			else await pending.dismiss(action.id);
			outcome = confirm ? 'confirmed' : 'dismissed';
		} catch (error) {
			failed(error, 'Réponse impossible');
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head><title>Demande d’un agent · Toile</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	<PageHeader title="Demande d’un agent" description="Un agent veut faire une modification qui demande votre accord." />
	{#if refused}
		<Alert tone="warning" class="flex flex-col items-start gap-3">
			<p>
				Pour confirmer, reconnectez-vous dans votre navigateur. L’app mobile ou de bureau, la CLI et les
				jetons d’API ne peuvent pas confirmer une demande d’agent.
			</p>
			{#if !isApp()}<Button variant="primary" onclick={signIn}>Se reconnecter</Button>{/if}
		</Alert>
	{/if}
	{#if action === undefined}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if outcome}
		<EmptyState
			icon={outcome === 'confirmed' ? icons.check : icons.close}
			title={outcome === 'confirmed' ? 'Modification appliquée' : 'Demande refusée'}
			description={outcome === 'confirmed'
				? 'Les formes ont été supprimées du tableau.'
				: 'Le tableau reste tel quel.'}
		>
			{#if boardId && outcome === 'confirmed'}<Button href="/b/{boardId}">Ouvrir le tableau</Button>{/if}
		</EmptyState>
	{:else if !action || !approval || !status}
		{#if !refused}
			<EmptyState
				icon={icons.warning}
				title="Demande introuvable"
				description="Elle a déjà été confirmée, refusée ou a expiré, ou elle n’est pas à votre nom."
			>
				<Button href="/">Retour à l’accueil</Button>
			</EmptyState>
		{/if}
	{:else}
		<div class="flex flex-wrap items-center gap-3 text-fc-xs text-fc-fg-muted">
			<StatusDot tone={status.tone} label={status.label} />
			<span>Client {action.client_id}</span>
			<span>Expire le {formatDate(action.expires_at)}</span>
			{#if boardId}<a class="text-fc-fg underline" href="/b/{boardId}">Voir le tableau</a>{/if}
		</div>
		<ApprovalCard {approval} {busy} onApprove={() => answer(true)} onDecline={() => answer(false)} />
	{/if}
</div>
