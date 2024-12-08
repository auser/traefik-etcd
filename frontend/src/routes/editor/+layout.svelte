<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/configStore';
  import ConfigEditorLayout from '$lib/components/layouts/ConfigEditorLayout.svelte';
  import HostSection from '$lib/components/config/HostSection.svelte';
  import { Alert } from '$lib/components/ui/alert';
	import { ChevronLeft } from 'lucide-svelte';
	import LayoutHeader from '@/components/page/LayoutHeader.svelte';

  let loading = true  ;

  onMount(() => {
    if ($configStore.draft) {
      $configStore.showDraftDialog = true;
      loading = false;
    }
  });
</script>

<slot name="header">
  <!-- <LayoutHeader title="Configuration Editor" /> -->
</slot>
<main class="">  
  <slot />
</main>
