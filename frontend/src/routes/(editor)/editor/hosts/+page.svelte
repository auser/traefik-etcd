<!-- src/routes/editor/hosts/+page.svelte -->
<script lang="ts">
	import { pageStore } from '$lib/stores/pageStore';
	import { Component, Plus } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import HostList from '$lib/components/editor/hosts/HostList.svelte';
	import { configStore } from '$lib/stores/configStore';
	import { goto } from '$app/navigation';

	onMount(() => {
		pageStore.setTitle('Host Configuration');
		pageStore.setActions([
			{
				label: 'Add Host',
				icon: Plus as unknown as Component,
				onClick: () => goto('/editor/hosts/new')
			}
		]);
	});
</script>

<HostList
	hosts={$configStore.currentConfig?.hosts || []}
	onChange={(hosts) =>
		configStore.updateConfig({
			...$configStore.currentConfig,
			hosts
		})}
/>
