<script lang="ts">
  import { Plus, Trash2 } from 'lucide-svelte';
  import type { TraefikConfigVersion } from '$lib/types';
  import AreYouSure from '$lib/components/AreYouSure.svelte';

  export let config: TraefikConfigVersion | null;
  let configToDelete: TraefikConfigVersion | null = null;

  export let onSelect: (config: TraefikConfigVersion) => void;
  export let onDeleteConfig: (config: TraefikConfigVersion) => void;

  async function handleDeleteConfig() {
    onDeleteConfig(configToDelete);
    configToDelete = null;
  }
</script>

<div
class="p-4 border rounded hover:border-blue-500 hover:bg-blue-50 text-left transition-colors"
>
<div class="grid grid-cols-2">
  <div class="flex items-center gap-2 flex-row">
    <Plus class="w-4 h-4" />
    <span class="font-medium">{config?.name}</span>
  </div>
  <div class="flex items-center gap-2">
    <span class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
      v{config?.version}
    </span>
    <button
    class="text-red-500 hover:text-red-700 p-1 rounded hover:bg-red-50"
    on:click={() => configToDelete = config}
    >
    <Trash2 class="w-4 h-4" />
  </button>
  <button
  class="flex items-center justify-center gap-2 px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
  on:click={() => onSelect(config)}
  >
  <Plus class="w-4 h-4" />
  Use Config
</button>
</div>
</div>
<AreYouSure 
open={!!configToDelete}
title="Delete Config"
message="Are you sure you want to delete this config?"
onConfirm={() => handleDeleteConfig()}
onCancel={() => configToDelete = null}
/>
</div>
