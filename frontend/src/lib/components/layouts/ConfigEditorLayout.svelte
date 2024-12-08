<script lang="ts">
  import { page } from '$app/stores';
  import { configStore } from '$lib/stores/configStore';
  import ConfigSidebar from '$lib/components/layouts/config/ConfigSidebar.svelte';
	import { ChevronLeft, Save } from 'lucide-svelte';

  export let showSaveButton = true;
</script>

<div class="flex h-screen bg-gray-50">
  <!-- Left Sidebar -->
  <ConfigSidebar showSaveButton={showSaveButton} />

  <!-- Main Content -->
  <div class="flex-1 flex flex-col">
    <header class="bg-white border-b px-6 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <a href="/config-list" class="text-gray-600 hover:text-gray-900">
            <ChevronLeft class="w-5 h-5" />
          </a>
          <h2 class="text-xl font-semibold">
            {$configStore.configName || 'New Configuration'}
          </h2>
          {#if $configStore.currentVersion}
            <span class="text-sm bg-blue-100 text-blue-800 px-2 py-1 rounded">
              v{$configStore.currentVersion}
            </span>
          {/if}
        </div>

        {#if showSaveButton}
          <div class="flex items-center space-x-3">
            <button 
              class="px-4 py-2 text-gray-600 hover:text-gray-900"
              on:click={() => history.back()}
            >
              Cancel
            </button>
            <button 
              class="flex items-center px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
              disabled={!$configStore.hasUnsavedChanges}
              on:click={() => configStore.save()}
            >
              <Save class="w-4 h-4 mr-2" />
              Save Changes
            </button>
          </div>
        {/if}
      </div>
    </header>

    <main class="flex-1 overflow-auto p-6">
      <div class="max-w-4xl mx-auto">
        <slot />
      </div>
    </main>
  </div>
</div>