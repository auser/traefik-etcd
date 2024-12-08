<!-- src/routes/editor/middleware/+page.svelte -->
<script lang="ts">
	import { Plus } from 'lucide-svelte';
	import MiddlewareCard from '$lib/components/editor/middleware/MiddlewareCard.svelte';
	import { configStore } from '$lib/stores/configStore';
	import { onMount } from 'svelte';
	import { pageStore } from '@/stores/pageStore';
	import ContentContainerLayout from '@/components/layouts/ContentContainerLayout.svelte';
	import NewResourceContainer from '@/components/page/NewResourceContainer.svelte';

	export let title = 'Middleware Configuration Editor';

	onMount(() => {
		pageStore.setTitle('Middleware Configuration');
	});

	function addMiddleware() {
		const currentConfig = $configStore.currentConfig || {};
		const middlewares = currentConfig.middlewares || {};

		configStore.updateConfig({
			...currentConfig,
			middlewares: {
				...middlewares,
				[`new-middleware-${Object.keys(middlewares).length + 1}`]: {
					headers: {
						customRequestHeaders: {},
						customResponseHeaders: {}
					},
					protocol: 'http',
					forwardAuth: undefined
				}
			}
		});
	}

	function updateMiddleware(name: string, config: any) {
		const currentConfig = $configStore.currentConfig || {};
		const middlewares = currentConfig.middlewares || {};

		configStore.updateConfig({
			...currentConfig,
			middlewares: {
				...middlewares,
				[name]: config
			}
		});
	}

	function deleteMiddleware(name: string) {
		const currentConfig = $configStore.currentConfig || {};
		const middlewares = { ...currentConfig.middlewares };
		delete middlewares[name];

		configStore.updateConfig({
			...currentConfig,
			middlewares
		});
	}
</script>

<ContentContainerLayout>
	<!-- Middleware List -->
	{#if !$configStore.currentConfig?.middlewares || Object.keys($configStore.currentConfig.middlewares).length === 0}
		<NewResourceContainer>
			<h3 class="text-lg font-medium text-gray-900">No middlewares configured</h3>
			<p class="mt-2 text-gray-500">Add a middleware to get started.</p>
			<button
				class="mt-4 px-4 py-2 text-sm text-blue-500 hover:text-blue-600"
				on:click={addMiddleware}
			>
				Add your first middleware
			</button>
		</NewResourceContainer>
	{:else}
		<div class="space-y-4">
			{#each Object.entries($configStore.currentConfig.middlewares) as [name, config]}
				<MiddlewareCard
					{name}
					{config}
					onChange={(updatedConfig) => updateMiddleware(name, updatedConfig)}
					onDelete={() => deleteMiddleware(name)}
				/>
			{/each}
		</div>
	{/if}
</ContentContainerLayout>
