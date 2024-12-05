<!-- src/lib/components/config/ConfigEditor.svelte -->
<script lang="ts">
  import { configStore } from '$lib/stores/configStore';
  import VisualConfigEditor from './VisualConfigEditor.svelte';
  import ConfigActions from './ConfigActions.svelte';
  import VersionHistory from './VersionHistory.svelte';
  import DiffViewer from './DiffViewer.svelte';
  import ConfigHeader from './ConfigHeader.svelte';
  
  $: showDiff = $configStore.showDiff;
</script>

<div>
  <ConfigHeader />
  
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
    <div class="lg:col-span-2">
      <VisualConfigEditor
        config={$configStore.currentConfig}
        name={$configStore.configName}
        onChange={(config) => configStore.updateConfig(config)}
        onNameChange={(name) => configStore.updateName(name)}
      />

      <div class="mt-4">
        <ConfigActions
          loading={$configStore.loading}
          saving={$configStore.saving}
          disabled={!$configStore.hasUnsavedChanges}
          onSave={() => configStore.save()}
          onReset={() => configStore.reset()}
        />
      </div>
    </div>

    <div>
      <VersionHistory
        versions={$configStore.versions}
        currentVersion={$configStore.currentVersion}
      />
    </div>
  </div>

  {#if showDiff}
    <DiffViewer
      oldVersion={$configStore.diffOldVersion}
      newVersion={$configStore.diffNewVersion}
      onClose={() => configStore.closeDiff()}
    />
  {/if}

  <StatusMessages
    error={$configStore.error}
    success={$configStore.saveStatus}
  />
</div>