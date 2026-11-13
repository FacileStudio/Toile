<script lang="ts">
	import { Button, Icon, Tooltip, icons } from '@facile/muse';
	import { logoIcon } from '$lib/brand';

	type Slide = { title: string; body: string; icon: string };

	const SLIDES: Slide[] = [
		{
			title: 'Un tableau sans bords',
			body: 'Post-it, formes, flèches et dessin à main levée sur une toile infinie.',
			icon: logoIcon
		},
		{
			title: 'À plusieurs, en direct',
			body: 'Voyez les curseurs des autres et dessinez ensemble, au bureau comme en déplacement.',
			icon: icons.usersGroup
		},
		{
			title: 'Au doigt ou au stylet',
			body: 'Pincez pour zoomer, glissez à deux doigts, écrivez au stylet avec la pression.',
			icon: icons.edit
		}
	];

	let { onfinish }: { onfinish: () => void } = $props();

	let track = $state<HTMLElement | null>(null);
	let index = $state(0);

	const last = $derived(index === SLIDES.length - 1);

	function onscroll() {
		if (track) index = Math.round(track.scrollLeft / track.clientWidth);
	}

	function show(target: number) {
		track?.scrollTo({ left: target * track.clientWidth, behavior: 'smooth' });
	}

	function next() {
		if (last) onfinish();
		else show(index + 1);
	}
</script>

<div class="fixed inset-0 flex flex-col overflow-hidden bg-fc-page text-fc-fg">
	<header class="flex h-14 items-center justify-between px-5">
		<span class="flex items-center gap-2 text-fc-md font-semibold"><Icon icon={logoIcon} size={22} />Toile</span>
		{#if !last}
			<Button variant="ghost" size="sm" onclick={onfinish}>Passer</Button>
		{/if}
	</header>

	<div
		bind:this={track}
		{onscroll}
		class="flex min-h-0 flex-1 snap-x snap-mandatory overflow-x-auto overscroll-x-contain [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
		aria-roledescription="carrousel"
	>
		{#each SLIDES as slide, i (slide.title)}
			<section
				class="flex w-full shrink-0 snap-center flex-col items-center justify-center gap-8 px-8 text-center"
				aria-roledescription="diapositive"
				aria-label="{i + 1} sur {SLIDES.length}"
				aria-hidden={i !== index}
			>
				<div class="flex size-36 items-center justify-center rounded-[2rem] bg-fc-surface shadow-sm">
					<Icon icon={slide.icon} size={56} />
				</div>
				<div class="flex max-w-xs flex-col gap-3">
					<h1 class="text-fc-3xl leading-tight font-semibold tracking-tight">{slide.title}</h1>
					<p class="text-fc-md leading-relaxed text-fc-fg-muted">{slide.body}</p>
				</div>
			</section>
		{/each}
	</div>

	<footer class="flex flex-col items-center gap-6 px-6 pt-4 pb-6">
		<div class="flex items-center gap-1" role="tablist" aria-label="Diapositives">
			{#each SLIDES as slide, i (slide.title)}
				<Tooltip text="Diapositive {i + 1}" describe={false} class="contents">
					<button
						type="button"
						role="tab"
						aria-selected={i === index}
						aria-label="Aller à la diapositive {i + 1}"
						class="flex size-11 items-center justify-center"
						onclick={() => show(i)}
					>
						<span
							class="h-1.5 rounded-full transition-all duration-300 motion-reduce:transition-none {i === index
								? 'w-6 bg-fc-fg'
								: 'w-1.5 bg-fc-fg/25'}"
						></span>
					</button>
				</Tooltip>
			{/each}
		</div>
		<Button size="lg" class="w-full max-w-sm" iconRight={icons.arrow} onclick={next}>
			{last ? 'Commencer' : 'Suivant'}
		</Button>
	</footer>
</div>
