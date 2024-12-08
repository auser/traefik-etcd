<script lang="ts">
	import { Plus, X } from 'lucide-svelte';
	import { configStore } from '$lib/stores/configStore';
	import { onMount } from 'svelte';
	import { pageStore } from '@/stores/pageStore';

	export let middlewares: string[] = [];
	export let onChange: (middlewares: string[]) => void;

	let showAddDialog = false;
	let selectedMiddleware = '';

	$: availableMiddlewares = Object.keys($configStore.currentConfig?.middlewares || {}).filter(
		(m) => !middlewares.includes(m)
	);

	onMount(() => {
		pageStore.setTitle('Middleware Configuration');
		pageStore.setActions([
			{
				label: 'Add Middleware',
				icon: Plus,
				onClick: () => (showAddDialog = true)
			}
		]);
	});

	function addMiddleware() {
		if (selectedMiddleware) {
			onChange([...middlewares, selectedMiddleware]);
			selectedMiddleware = '';
			showAddDialog = false;
		}
	}

	function removeMiddleware(middleware: string) {
		onChange(middlewares.filter((m) => m !== middleware));
	}
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<h4 class="font-medium">Middlewares</h4>
		{#if availableMiddlewares.length > 0}
			<button
				class="text-sm text-blue-500 hover:text-blue-600"
				on:click={() => (showAddDialog = true)}
			>
				<Plus class="h-4 w-4" />
			</button>
		{/if}
	</div>

	<div class="flex flex-wrap gap-2">
		{#if middlewares.length === 0}
			<p class="text-sm text-gray-500">No middlewares attached</p>
		{:else}
			{#each middlewares as middleware}
				<div class="flex items-center gap-1 rounded bg-gray-100 px-2 py-1 text-sm">
					<span>{middleware}</span>
					<button
						class="text-gray-400 hover:text-red-500"
						on:click={() => removeMiddleware(middleware)}
					>
						<X class="h-3 w-3" />
					</button>
				</div>
			{/each}
		{/if}
	</div>

	{#if showAddDialog}
		<div class="mt-2 rounded border bg-gray-50 p-3">
			<div class="space-y-3">
				<h5 class="text-sm font-medium">Add Middleware</h5>
				<select class="w-full rounded-md border-gray-300 text-sm" bind:value={selectedMiddleware}>
					<option value="">Select middleware...</option>
					{#each availableMiddlewares as middleware}
						<option value={middleware}>{middleware}</option>
					{/each}
				</select>
				<div class="flex justify-end gap-2">
					<button
						class="px-2 py-1 text-sm text-gray-600 hover:text-gray-900"
						on:click={() => (showAddDialog = false)}
					>
						Cancel
					</button>
					<button
						class="rounded bg-blue-500 px-2 py-1 text-sm text-white hover:bg-blue-600 disabled:opacity-50"
						disabled={!selectedMiddleware}
						on:click={addMiddleware}
					>
						Add
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
