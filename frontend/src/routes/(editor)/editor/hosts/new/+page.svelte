<script lang="ts">
	import HostEditor from '$lib/components/editor/hosts/HostEditor.svelte';
	import { pageStore } from '@/stores/pageStore';
	import { onMount } from 'svelte';

	onMount(() => {
		pageStore.setTitle('New Host');
	});
</script>

<HostEditor />
