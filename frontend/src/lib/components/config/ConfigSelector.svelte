<!-- src/lib/components/config/ConfigSelector.svelte -->
<script lang="ts">
  import type { ConfigListItem } from '$lib/stores/configStore';
  
  export let configs: ConfigListItem[] = [];
  export let selectedId: number | null = null;
  export let loading = false;
  export let saving = false;

  export let onSelect: (id: number) => void;
  export let onNew: () => void;
</script>

<div class="flex gap-2 items-center mb-2">
  <select 
    class="flex-1 p-2 border rounded"
    value={selectedId ?? ''}
    on:change={(e) => onSelect(parseInt(e.currentTarget.value))}
    disabled={loading || saving}
  >
    <option value="">Select a configuration</option>
    {#each configs as config}
      <option value={config.id}>
        {config.name}
        {#if config.source}
          ({config.source})
        {/if}
      </option>
    {/each}
  </select>

  <button 
    class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
    on:click={onNew}
    disabled={loading || saving}
  >
    New Config
  </button>
</div>