<!-- src/lib/components/config/TemplateSelector.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Clock, Trash2 } from 'lucide-svelte';
  import { debounce } from 'lodash';
  import { fetching } from '$lib/utils/fetching';

  export let configId: number;
  export let onDeleteConfig: () => void;

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleString();
  }
</script>

<div class="space-y-4">
  <h3 class="text-xl font-semibold">Delete Configuration</h3>
  

  <!-- Delete Confirmation Dialog -->
  {#if configId}
    <AlertDialog open={true}>
      <AlertDialogContent>
        <h2 class="text-lg font-semibold text-red-600">Delete Configuration</h2>
        <p class="mt-2">
          Are you sure you want to delete "{configName}"? This action cannot be undone.
        </p>
        <div class="flex justify-end gap-2 mt-4">
          <button
            class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded"
            on:click={() => configId = null}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
            on:click={onDeleteConfig}
          >
            Delete
          </button>
        </div>
      </AlertDialogContent>
    </AlertDialog>
  {/if}
</div>