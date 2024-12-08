<!-- src/lib/components/config/TemplateSelector.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/configStore';
  import { Search, Plus, Trash2 } from 'lucide-svelte';
  import { debug } from '$lib/utils/logging';
  import AreYouSure from '$lib/components/AreYouSure.svelte';
  import type { TemplateInfo, TraefikConfigVersion } from '$lib/types';
  import { fetching } from '$lib/utils/fetching';
  import FilteredConfigOrTemplateItem from './FilteredConfigOrTemplateItem.svelte';

  let loading = false;
  let error = '';
  let searchTerm = '';
  let templates: TemplateInfo[] = [];
  let configs: TraefikConfigVersion[] = [];
  let filteredTemplates: TemplateInfo[] = [];
  let filteredConfigs: TraefikConfigVersion[] = [];
  let templateToDelete: TemplateInfo | null = null;

  onMount(async () => {
    loading = true;
    try {
      templates = await configStore.loadTemplates();
      configs = await configStore.loadConfigs();
      filteredConfigs = configs;
    } catch (e) {
      debug('Failed to load templates:', e);
      error = 'Failed to load templates';
    } finally {
      loading = false;
    }
  });

  $: {
    if (templateToDelete) {
      console.log('templateToDelete', templateToDelete);
    }
  }

  $: {
    if (searchTerm.trim()) {
      console.log('Searching for templates', searchTerm);
      const term = searchTerm.toLowerCase();
      filteredConfigs = configs.filter(t => 
        t.name.toLowerCase().includes(term)
      );
    } else {
      console.log('No search term, showing all templates');
      filteredConfigs = templates;
    }
  }

  async function handleSelectConfig(template: TemplateInfo) {
    try {
      await configStore.selectTemplate(template);
    } catch (e) {
      debug('Failed to select template:', e);
      error = 'Failed to load template';
    }
  }

  async function handleDeleteTemplate(template: TemplateInfo) {
    try {
      await configStore.deleteTemplate(template);
      templateToDelete = null;
      templates = await configStore.loadTemplates();
      configs = await configStore.loadConfigs();
    } catch (e) {
      debug('Failed to delete template:', e);
      error = 'Failed to delete template';
    }
  }
</script>

<div class="space-y-4">
  <h3 class="text-xl font-semibold">Select Template</h3>
  
  <div class="relative">
    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
      <Search class="w-4 h-4 text-gray-400" />
    </div>
    <input
      type="text"
      bind:value={searchTerm}
      placeholder="Search templates..."
      class="w-full pl-10 pr-4 py-2 border rounded"
    />
  </div>

  {#if loading}
    <div class="text-center py-4">Loading templates...</div>
  {:else if error}
    <div class="text-red-500">{error}</div>
  {:else if filteredConfigs.length === 0}
    <div class="text-center py-8 text-gray-500">
      No templates found matching "{searchTerm}"
    </div>
  {:else if searchTerm.trim()}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredConfigs as config (config.id)}
        <FilteredConfigOrTemplateItem config={config} onSelect={handleSelectConfig} onDeleteConfig={handleDeleteTemplate} />
        <!-- <div
          class="p-4 border rounded hover:border-blue-500 hover:bg-blue-50 text-left transition-colors"
        >
          <div class="grid grid-cols-2">
            <div class="flex items-center gap-2 flex-row">
              <Plus class="w-4 h-4" />
              <span class="font-medium">{template.name}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                v{template.version}
              </span>
              <button
              class="text-red-500 hover:text-red-700 p-1 rounded hover:bg-red-50"
              on:click={() => templateToDelete = template}
              >
              <Trash2 class="w-4 h-4" />
            </button>
            <button
            class="flex items-center justify-center gap-2 px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
            on:click={() => handleSelectTemplate(template)}
            >
            <Plus class="w-4 h-4" />
            Use Template
          </button>
        </div>
      </div> -->
    <!-- </div> -->
      {/each}
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each templates as config (config.id)}
        <FilteredConfigOrTemplateItem config={config} onSelect={handleSelectConfig} onDeleteConfig={handleDeleteTemplate} />
      {/each}
    </div>
  {/if}


</div>
