<!-- src/routes/config-editor/+page.svelte -->
<script lang="ts">
    import { onMount } from 'svelte';
    import { configStore } from '$lib/stores/configStore';
    import { Alert } from '$lib/components/ui/alert';
    import { Save, FileDown, AlertCircle } from 'lucide-svelte';
  
    onMount(() => {
      configStore.fetchConfigs();
    });
  </script>
  
  <div class="p-4 max-w-4xl mx-auto">
    <div class="mb-4">
      <h2 class="text-2xl font-bold mb-4">Configuration Editor</h2>
      
      <!-- Config Selection -->
      <div class="mb-4">
        <select 
          class="w-full p-2 border rounded"
          on:change={(e) => configStore.loadConfig(e.currentTarget.value)}
          value={$configStore.selectedConfig?.id || ''}
        >
          <option value="">Select a configuration</option>
          {#each $configStore.configs as config}
            <option value={config.id}>
              {config.name}
            </option>
          {/each}
        </select>
      </div>
  
      <!-- Editor -->
      {#if $configStore.selectedConfig}
        <div class="mb-4">
          <textarea
            class="w-full h-96 p-4 font-mono text-sm border rounded"
            value={$configStore.editedContent}
            on:input={(e) => configStore.updateEditedContent(e.currentTarget.value)}
            spellcheck="false"
          />
        </div>
      {/if}
  
      <!-- Actions -->
      <div class="flex gap-2">
        <button
          class="px-4 py-2 bg-blue-500 text-white rounded flex items-center gap-2 hover:bg-blue-600 disabled:opacity-50"
          on:click={() => configStore.saveConfig()}
          disabled={!$configStore.selectedConfig}
        >
          <Save class="w-4 h-4" />
          Save
        </button>
        <button
          class="px-4 py-2 bg-gray-500 text-white rounded flex items-center gap-2 hover:bg-gray-600 disabled:opacity-50"
          on:click={() => configStore.reset()}
          disabled={!$configStore.selectedConfig}
        >
          <FileDown class="w-4 h-4" />
          Reset
        </button>
      </div>
  
      <!-- Error Display -->
      {#if $configStore.error}
        <Alert variant="destructive" class="mt-4">
          <AlertCircle class="h-4 w-4" />
          <svelte:fragment slot="title">Error</svelte:fragment>
          <svelte:fragment slot="description">{$configStore.error}</svelte:fragment>
        </Alert>
      {/if}
  
      <!-- Save Status -->
      {#if $configStore.saveStatus}
        <Alert class="mt-4 bg-green-50">
          <svelte:fragment slot="title">Success</svelte:fragment>
          <svelte:fragment slot="description">{$configStore.saveStatus}</svelte:fragment>
        </Alert>
      {/if}
    </div>
  </div>