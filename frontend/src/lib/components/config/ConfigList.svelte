<!-- src/lib/components/config/ConfigList.svelte -->
<script lang="ts">
  import { Trash2 } from 'lucide-svelte';
  import type { ConfigListItem } from '$lib/stores/configStore';
  import { configStore } from '$lib/stores/configStore';
  import { fetching } from '$lib/utils/fetching';
  export let configs: ConfigListItem[] = [];
  let confirmDelete: number | null = null;

  async function deleteConfig(id: number) {
    try {
      const response = await fetching(`/configs/delete/${id}`, {
        method: "DELETE"
      });
      
      if (!response.ok) throw new Error('Failed to delete');
      
      // Refresh the configs list
      await configStore.fetchConfigs();
    } catch (error) {
      console.error('Failed to delete config:', error);
    }
  }
</script>

<div class="space-y-4">
  <h3 class="text-lg font-semibold">Configurations</h3>
  
  <div class="border rounded divide-y">
    {#each configs as config (config.id)}
      <div class="p-4 flex items-center justify-between hover:bg-gray-50">
        <div>
          <h4 class="font-medium">{config.name}</h4>
          <p class="text-sm text-gray-500">
            Last updated: {new Date(config.updated_at).toLocaleString()}
          </p>
        </div>
        
        <div class="flex items-center gap-2">
          {#if confirmDelete === config.id}
            <div class="flex items-center gap-2">
              <span class="text-sm">Are you sure?</span>
              <button
                class="px-2 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
                on:click={() => deleteConfig(config.id)}
              >
                Yes, delete
              </button>
              <button
                class="px-2 py-1 text-sm bg-gray-500 text-white rounded hover:bg-gray-600"
                on:click={() => confirmDelete = null}
              >
                Cancel
              </button>
            </div>
          {:else}
            <button
              class="text-red-500 hover:text-red-700"
              on:click={() => confirmDelete = config.id}
            >
              <Trash2 class="w-4 h-4" />
            </button>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>