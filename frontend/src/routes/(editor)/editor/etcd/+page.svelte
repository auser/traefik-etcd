<script lang="ts">
	import EtcdConfig from '$lib/components/config/EtcdConfig.svelte';
	import { pageStore } from '$lib/stores/pageStore';
	import ContentContainerLayout from '$lib/components/layouts/ContentContainerLayout.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		pageStore.setTitle('ETCD Configuration Editor');
	});
</script>

<ContentContainerLayout>
	<EtcdConfig />
</ContentContainerLayout>
