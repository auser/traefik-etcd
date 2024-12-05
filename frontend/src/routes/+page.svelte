<!-- src/routes/config-editor/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/configStore';
  import YAML, { parse as parseYAML } from 'yaml';
  import type { TraefikConfig } from '$lib/types';
  
  import ConfigSelector from '$lib/components/config/ConfigSelector.svelte';
  import VisualConfigEditor from '$lib/components/config/VisualConfigEditor.svelte';
  import ConfigActions from '$lib/components/config/ConfigActions.svelte';
  import StatusMessages from '$lib/components/config/StatusMessages.svelte';
  import ConfigList from '$lib/components/config/ConfigList.svelte';
  import TemplateSelector from '$lib/components/config/TemplateSelector.svelte';

  let loading = false;
  let saving = false;
  let error = '';
  let currentConfig: TraefikConfig | null = null;
  let selectedTemplate: TemplateInfo | null = null;

  onMount(async () => {
    loading = true;
    try {
      await configStore.loadConfigs();
    } finally {
      loading = false;
    }
  });

  // Parse the edited content whenever it changes
  $: {
    try {
      if ($configStore.editedContent) {
        // Parse YAML content into TraefikConfig
        currentConfig = parseYAML($configStore.editedContent);
        error = '';
      } else {
        currentConfig = null;
      }
    } catch (e) {
      error = e.message;
      console.error('Parse error:', e); // For debugging
    }
  }

  async function handleConfigSelect(id: number) {
    loading = true;
    try {
      let res = await configStore.selectConfig(id);
    } finally {
      loading = false;
    }
  }

  async function handleTemplateSelect(template) {
    try {
      await configStore.loadTemplate(template.name);
      currentConfig = $configStore.template;
      selectedTemplate = template;
    } catch (e) {
      error = 'Failed to load template';
    }
  }

  async function handleSave() {
    if (!currentConfig) return;

    saving = true;
    try {
      // Convert config back to YAML before saving
      const yamlContent = YAML.stringify(currentConfig);
      configStore.updateEditedContent(yamlContent);
      await configStore.saveConfig();
    } finally {
      saving = false;
    }
  }

  function handleConfigChange(updatedConfig: TraefikConfig) {
    currentConfig = {...currentConfig, updatedConfig};
  }
</script>

<div class="p-4 max-w-4xl mx-auto">
  <div>{!currentConfig ? 'no config' : 'has config'}</div>
  {#if !currentConfig}
    <TemplateSelector onSelectTemplate={handleTemplateSelect} />
  {:else}
    <div class="mb-4">
      <h2 class="text-2xl font-bold">
        New Configuration
        <span class="text-sm font-normal text-gray-500">
          (based on {selectedTemplate.name})
        </span>
      </h2>
    </div>
  <div class="mb-4">
    <h2 class="text-2xl font-bold mb-4">Configuration Editor</h2>

    <ConfigSelector
      configs={$configStore.configs}
      selectedId={$configStore.selectedId}
      {loading}
      {saving}
      onSelect={handleConfigSelect}
      onNew={() => handleConfigSelect(0)}
    />

    {#if currentConfig}
      <div class="mt-4">
        <VisualConfigEditor
          config={currentConfig}
          onChange={handleConfigChange}
          onNameChange={(newName) => configStore.updateName(newName)}
        />
      </div>

      <div class="mt-4">
        <ConfigActions
          {loading}
          {saving}
          disabled={!$configStore.selectedId}
          onSave={handleSave}
          onReset={() => configStore.reset()}
        />
      </div>
    {:else if $configStore.editedContent}
      <div class="text-center py-4 text-red-500">
        Error parsing configuration data
      </div>
    {/if}

    {#if loading}
      <div class="text-center py-4">
        Loading...
      </div>
    {/if}

    <StatusMessages
      error={error || $configStore.error}
        success={$configStore.saveStatus}
      />
    </div>
  {/if}
</div>