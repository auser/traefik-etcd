<!-- src/routes/config-editor/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/configStore';
  import TemplateSelector from '$lib/components/config/TemplateSelector.svelte';
  import VisualConfigEditor from '$lib/components/config/VisualConfigEditor.svelte';
  import ConfigActions from '$lib/components/config/ConfigActions.svelte';
  import StatusMessages from '$lib/components/config/StatusMessages.svelte';
  import VersionHistory from '$lib/components/config/VersionHistory.svelte';
  import DiffViewer from '$lib/components/config/DiffViewer.svelte';
  import AreYouSure from '$lib/components/AreYouSure.svelte';
  
  let deleteConfigDialogOpen = false;

  onMount(() => {
    if ($configStore.draft) {
      $configStore.showDraftDialog = true;
    }
  });
</script>

<div class="p-4 max-w-4xl mx-auto">
  {#if $configStore.showDraftDialog}
    <AreYouSure
      open={true}
      title="Resume Editing?"
      message="You have an unsaved draft from {new Date($configStore.draft.lastSaved).toLocaleString()}. Would you like to continue editing?"
      onConfirm={() => configStore.discardDraft()}
    />
    <!-- <AlertDialog open={true}>
      <AlertDialogContent>
        <h2 class="text-lg font-semibold">Resume Editing?</h2>
        <p class="mt-2">
          You have an unsaved draft from {new Date($configStore.draft.lastSaved).toLocaleString()}.
          Would you like to continue editing?
        </p>
        <div class="flex justify-end gap-2 mt-4">
          <button
            class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded"
            on:click={() => configStore.discardDraft()}
          >
            Discard
          </button>
          <button
            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
            on:click={() => configStore.loadDraft()}
          >
            Resume Editing
          </button>
        </div>
      </AlertDialogContent>
    </AlertDialog> -->
  {/if}

  {#if !$configStore.currentConfig}
    <TemplateSelector />
  {:else}
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
          on:click={() => configStore.reset()}
        >
          <small>back</small>
        </button>
        <button 
          class="text-sm font-normal text-gray-500" 
          on:click={() => deleteConfigDialogOpen = true}
        >
          <small>delete</small>
        </button>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <div class="lg:col-span-2">
        <VisualConfigEditor
          config={$configStore.currentConfig}
          configName={$configStore.configName}
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

    <AreYouSure 
      open={deleteConfigDialogOpen}
      title="Delete Configuration" 
      message="Are you sure you want to delete this configuration?"
      onConfirm={() => {
        configStore.deleteCurrentConfig();
        deleteConfigDialogOpen = false;
      }}
    />

    {#if $configStore.showDiff}
      <DiffViewer
        oldVersion={$configStore.diffOldVersion}
        newVersion={$configStore.diffNewVersion}
      />
    {/if}

    <StatusMessages 
      error={$configStore.error} 
      success={$configStore.saveStatus} 
    />
  {/if}
</div>