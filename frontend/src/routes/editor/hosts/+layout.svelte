<script lang="ts"> 
	import LayoutHeader from '@/components/page/LayoutHeader.svelte';
	import ConfigEditorLayout from '@/components/layouts/ConfigEditorLayout.svelte';
</script>

<slot name="header">
  <LayoutHeader title="Hosts Configuration Editor" />
</slot>

<ConfigEditorLayout>
  <slot />
</ConfigEditorLayout>