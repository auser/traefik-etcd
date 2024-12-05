<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/configStore';
  import YAML, { parse as parseYAML } from 'yaml';
  import type { TraefikConfig } from '$lib/types';
  
  import ConfigList from '$lib/components/config/ConfigList.svelte';

  onMount(async () => {
    loading = true;
    await configStore.loadConfigs();
    loading = false;
  });
</script>

<!-- Configuration List -->
<div>
  <ConfigList configs={$configStore.configs} />
</div>