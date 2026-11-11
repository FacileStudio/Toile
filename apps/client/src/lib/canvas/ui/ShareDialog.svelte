<script lang="ts">
	import { Button, IconButton, Icon, Modal, Select, icons, toast } from '@facile/muse';
	import { boardApi, type NewShare, type Share, type ShareRole } from '$lib/board/api';
	import { formatDate } from '$lib/format';

	let { board, open = $bindable(false) }: { board: string; open?: boolean } = $props();

	const ROLES: { value: ShareRole; label: string }[] = [
		{ value: 'view', label: 'Lecture seule' },
		{ value: 'comment', label: 'Commentaire' },
		{ value: 'edit', label: 'Modification' }
	];
	const roleLabel = (role: ShareRole) => ROLES.find((r) => r.value === role)?.label ?? role;

	let shares = $state<Share[]>([]);
	let fresh = $state<NewShare | null>(null);
	let role = $state<string>('view');
	let busy = $state(false);

	const linkOf = (share: NewShare) => new URL(share.url || `/s/${share.token}`, location.origin).toString();

	async function attempt(action: () => Promise<void>) {
		busy = true;
		try {
			await action();
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Action impossible');
		} finally {
			busy = false;
		}
	}

	$effect(() => {
		if (!open) return;
		fresh = null;
		void attempt(async () => void (shares = await boardApi.shares(board)));
	});

	const create = () =>
		attempt(async () => {
			const share = await boardApi.share(board, role as ShareRole);
			shares = [...shares, share];
			fresh = share;
			await copy(share);
		});

	const revoke = (share: Share) =>
		attempt(async () => {
			await boardApi.revoke(board, share.id);
			shares = shares.filter((s) => s.id !== share.id);
			if (fresh?.id === share.id) fresh = null;
		});

	async function copy(share: NewShare) {
		try {
			await navigator.clipboard.writeText(linkOf(share));
			toast.success('Lien copié.');
		} catch {
			toast.info('Copiez le lien affiché.');
		}
	}
</script>

<Modal bind:open title="Partager le tableau" showClose>
	<div class="flex flex-col gap-5">
		<p class="text-fc-sm text-fc-fg-muted">
			Les membres de l’espace y ont déjà accès. Un lien ouvre le tableau à toute personne qui le reçoit, sans compte,
			jusqu’à ce que vous le révoquiez. Il n’est affiché qu’une fois : copiez-le maintenant.
		</p>
		<div class="flex flex-wrap items-end gap-2">
			<Select bind:value={role} options={ROLES} aria-label="Droit du lien" class="min-w-0 flex-1" />
			<Button icon={icons.plus} disabled={busy} onclick={create}>Nouveau lien</Button>
		</div>
		{#if fresh}
			<div class="flex items-center gap-2 rounded-fc-md bg-fc-surface py-1 pr-1 pl-3">
				<input
					class="min-w-0 flex-1 bg-transparent font-fc-mono text-fc-xs text-fc-fg outline-none"
					readonly
					value={linkOf(fresh)}
					aria-label="Lien {roleLabel(fresh.role)}"
					onfocus={(e) => e.currentTarget.select()}
				/>
				<IconButton variant="ghost" aria-label="Copier le lien" onclick={() => fresh && copy(fresh)}>
					<Icon icon={icons.copy} size={18} />
				</IconButton>
			</div>
		{/if}
		{#if shares.length}
			<ul class="flex flex-col gap-2" aria-label="Liens de partage">
				{#each shares as share (share.id)}
					<li class="flex items-center gap-2 py-1 pl-3">
						<span class="min-w-0 flex-1 truncate text-fc-sm text-fc-fg">{roleLabel(share.role)}</span>
						{#if share.created_at}<span class="shrink-0 text-fc-xs text-fc-fg-muted"
								>{formatDate(share.created_at)}</span
							>{/if}
						<IconButton
							variant="ghost-danger"
							aria-label="Révoquer le lien {roleLabel(share.role)}"
							onclick={() => revoke(share)}
						>
							<Icon icon={icons.revoke} size={18} />
						</IconButton>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</Modal>
