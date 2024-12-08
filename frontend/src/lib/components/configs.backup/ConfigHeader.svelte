<!-- src/lib/components/config/ConfigHeader.svelte -->
<script lang="ts">
  import { configStore } from '$lib/stores/configStore';
  import AreYouSure from '../AreYouSure.svelte';
  
  let deleteDialogOpen = false;
</script>

<div class="mb-4 flex items-center justify-between">
  <h2 class="text-2xl font-bold">
    {$configStore.configId ? 'Edit Configuration' : 'New Configuration'}
    {#if $configStore.selectedTemplate}
      <span class="text-sm font-normal text-gray-500">
        (based on {$configStore.selectedTemplate.name})
      </span>
    {/if}
  </h2>
  <div class="flex items-center gap-2">
    <span class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
      v{$configStore.currentVersion}
    </span>
    <button
      class="text-sm font-normal text-gray-500"
      on:click={() => {
        configStore.clearDraft();
        configStore.reset();
      }}
    >
      <small>back</small>
    </button>
    <button
      class="text-sm font-normal text-gray-500"
      on:click={() => deleteDialogOpen = true}
    >
      <small>delete</small>
    </button>
  </div>

  <AreYouSure
    open={deleteDialogOpen}
    title="Delete Configuration"
    message="Are you sure you want to delete this configuration?"
    onConfirm={() => configStore.deleteCurrentConfig()}
  />
</div>