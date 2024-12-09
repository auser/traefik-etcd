<script lang="ts">
	import { onMount } from 'svelte';
	import { configStore } from '$lib/stores/configStore';
	import ContentContainerLayout from '$lib/components/layouts/ContentContainerLayout.svelte';
	import ServiceList from '@/components/editor/services/ServiceList.svelte';
	import ServiceDetails from '@/components/editor/services/ServiceDetails.svelte';
	import { pageStore } from '@/stores/pageStore';
	import LayoutHeader from '@/components/page/LayoutHeader.svelte';
	import { goto } from '$app/navigation';
	import { Plus } from 'lucide-svelte';

	let lastSavedMessage = '';

	$: if ($configStore.lastSaved) {
		lastSavedMessage = `Last saved: ${$configStore.lastSaved.toLocaleString()}`;
	}

	onMount(() => {
		pageStore.setTitle('Edit Services');
		pageStore.setActions([
			{
				label: 'Add Service',
				icon: Plus,
				onClick: () => goto('/editor/services/new')
			}
		]);
		pageStore.setLastSavedMessage(lastSavedMessage);
		console.log('currentConfig', $configStore.currentConfig);
		if (!$configStore.currentConfig) {
			// window.location.href = '/config-list';
			goto('/config-list');
		}
	});

	function updateServices(services: Record<string, any>) {
		configStore.updateCurrentConfig({
			http: {
				...$configStore.currentConfig?.http,
				services
			}
		});
	}
</script>

<ContentContainerLayout>
	<div class="mb-6">
		<div class="service-list mb-6">
			<ServiceList />
		</div>
	</div>
</ContentContainerLayout>
