<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus } from 'lucide-svelte';
  import { configStore } from '$lib/stores/configStore';
  interface TemplateInfo {
    name: string;
    path: string;
    description?: string;
  }

  export let onSelectTemplate: (template: TemplateInfo) => void;
  let templates: TemplateInfo[] = [];
  let loading = false;
  let error = '';

  onMount(async () => {
    loading = true;
    try {
      await configStore.loadTemplates();
      templates = $configStore.templates;
    } catch (e) {
      error = 'Failed to load templates';
    } finally {
      loading = false;
    }
  });
</script>

<div class="space-y-4">
  <h3 class="text-xl font-semibold">Create New Configuration</h3>
  {#if loading}
    <div class="text-center py-4">Loading templates...</div>
  {:else if error}
    <div class="text-red-500">{error}</div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each templates as template}
        <button
          class="p-4 border rounded hover:border-blue-500 hover:bg-blue-50 text-left transition-colors"
          on:click={() => onSelectTemplate(template)}
        >
          <div class="flex items-center gap-2">
            <Plus class="w-4 h-4" />
            <span class="font-medium">{template.name}</span>
          </div>
          {#if template.description}
            <p class="mt-2 text-sm text-gray-600">{template.description}</p>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>