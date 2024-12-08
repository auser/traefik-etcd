<script lang="ts">
	import type { PageAction } from '$lib/stores/pageStore';
	export let title: string;
	export let actions: PageAction[] = [];
</script>

<header>
	<div class="flex items-center justify-between">
		<div class="flex items-center space-x-4">
			<h2 class="text-xl font-semibold">{title}</h2>
		</div>
		{#each actions as action}
			<button
				class="flex items-center gap-2 rounded-md bg-blue-500 px-3 py-2 text-sm text-white hover:bg-blue-600"
				on:click={action.onClick}
			>
				{#if action.icon}
					<svelte:component this={action.icon} class="h-4 w-4" />
				{/if}

				{action.label}
			</button>
		{/each}
	</div>
</header>
