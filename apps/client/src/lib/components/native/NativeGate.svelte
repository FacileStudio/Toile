<script lang="ts">
	import NativeIntro from './NativeIntro.svelte';
	import NativeLogin from './NativeLogin.svelte';

	const SEEN_KEY = 'toile.intro-seen';

	let { onsignedin }: { onsignedin: () => Promise<void> } = $props();

	function seen(): boolean {
		try {
			return localStorage.getItem(SEEN_KEY) === '1';
		} catch {
			return false;
		}
	}

	let intro = $state(!seen());

	function finish() {
		intro = false;
		try {
			localStorage.setItem(SEEN_KEY, '1');
		} catch {
			return;
		}
	}
</script>

{#if intro}
	<NativeIntro onfinish={finish} />
{:else}
	<NativeLogin {onsignedin} />
{/if}
