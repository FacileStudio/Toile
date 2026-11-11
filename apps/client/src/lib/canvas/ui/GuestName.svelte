<script lang="ts">
	import { Button, Card, Field, Input } from '@facile/muse';
	import { rememberGuest } from '$lib/board/guest';
	import { logoIcon } from '$lib/brand';
	import { Icon } from '@facile/muse';

	let { name = $bindable(''), board, onconfirm }: { name?: string; board: string; onconfirm: () => void } = $props();

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const kept = rememberGuest(name);
		if (!kept) return;
		name = kept;
		onconfirm();
	}
</script>

<div class="flex min-h-dvh items-center justify-center bg-fc-page p-4">
	<Card class="flex w-full max-w-sm flex-col gap-5">
		<div class="flex items-center gap-2 text-fc-fg">
			<Icon icon={logoIcon} size={24} /><span class="font-semibold">Toile</span>
		</div>
		<div class="flex flex-col gap-1">
			<h1 class="text-fc-xl font-semibold text-fc-fg">{board}</h1>
			<p class="text-fc-sm text-fc-fg-muted">Choisissez le nom que les autres verront à côté de votre curseur.</p>
		</div>
		<form class="flex flex-col gap-3" onsubmit={submit}>
			<Field label="Votre nom">
				<Input bind:value={name} maxlength={40} required autocomplete="nickname" placeholder="Camille" />
			</Field>
			<Button type="submit" disabled={!name.trim()}>Ouvrir le tableau</Button>
		</form>
	</Card>
</div>
