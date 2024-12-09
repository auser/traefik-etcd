<script lang="ts">
	import { configStore } from '$lib/stores/configStore';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { pageStore } from '@/stores/pageStore';
	import NewResourceContainer from '@/components/page/NewResourceContainer.svelte';

	onMount(() => {
		pageStore.setTitle('Services');
		pageStore.setActions([
			{
				label: 'Add Services',
				onClick: addNewService
			}
		]);
		configStore.loadConfigs();
	});

	$: services = $configStore.currentConfig?.config?.services || {};

	function addNewService() {
		const newService = {
			loadBalancer: {
				servers: [{ url: 'http://localhost' }]
			}
		};
		console.log('services', services);
		const newServiceName = `service-${Object.keys(services).length + 1}`;
		const newServices = {
			...services,
			[newServiceName]: newService
		};

		configStore.updateCurrentConfig({
			[newServiceName]: newService
		});
	}
</script>

<div class="services-list">
	<div class="service-items">
		{#if Object.keys(services).length === 0}
			<NewResourceContainer>
				<div class="rounded py-8 text-center text-gray-500">
					No services configured. Click "Add Service" to begin.
				</div>
			</NewResourceContainer>
		{:else}
			{#each Object.keys(services) as serviceName}
				<div class="service-item">
					<div class="service-info">
						<h3 class="font-medium">{serviceName}</h3>
						<p class="text-sm text-gray-500">
							URL config
							<!-- {services[serviceName].loadBalancer?.servers?.[0]?.url || 'No URL configured'} -->
						</p>
					</div>
					<button
						class="edit-btn"
						on:click={() => goto(`/editor/hosts/${encodeURIComponent(serviceName)}`)}
					>
						Edit
					</button>
				</div>
			{/each}
		{/if}
	</div>
</div>

<!-- <div class="service-details">
  <ServiceDetails />
</div> -->
