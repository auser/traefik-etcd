<script lang="ts">
	import HostEditor from '$lib/components/editor/hosts/HostEditor.svelte';
	import { page } from '$app/stores';
</script>

<HostEditor hostId={$page.params.id} />
