<script lang="ts">
	import ConfigEditorLayout from '@/components/layouts/ConfigEditorLayout.svelte';
	import LayoutHeader from '@/components/page/LayoutHeader.svelte';
	import { pageStore } from '$lib/stores/pageStore';
</script>

<ConfigEditorLayout>
	<slot name="header">
		<LayoutHeader title={$pageStore.title} actions={$pageStore.actions} />
	</slot>
	<main class="">
		<slot />
	</main>
</ConfigEditorLayout>
