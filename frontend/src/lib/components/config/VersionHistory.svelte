<script lang="ts">
  import { configStore } from '$lib/stores/configStore';
  import { Clock } from 'lucide-svelte';

  export let versions: any[] = [];
  export let currentVersion: number;

  $: sortedVersions = [...versions].sort((a, b) => b.version - a.version);

  function formatDate(date: string) {
    return new Date(date).toLocaleString();
  }

  function handleShowDiff(version: any) {
    const currentVersionData = sortedVersions.find(v => v.version === currentVersion - 1);
    if (currentVersionData) {
      configStore.showDiff(version, currentVersionData);
    }
  }
</script>

<div class="border rounded-lg overflow-hidden">
  <div class="bg-gray-50 p-4 border-b">
    <h3 class="font-semibold">Version History</h3>
  </div>
  <div class="divide-y">
    {#each sortedVersions as version}
      <div 
        class="p-4 hover:bg-gray-50 flex items-center justify-between"
        class:bg-blue-50={version.version === currentVersion}
      >
        <div>
          <div class="font-medium">Version {version.version}</div>
          <div class="text-sm text-gray-500 flex items-center gap-1">
            <Clock class="w-3 h-3" />
            {formatDate(version.created_at)}
          </div>
        </div>
        {#if version.version !== currentVersion}
          <div class="flex gap-2">
            <button
              class="text-blue-600 hover:text-blue-800 text-sm"
              on:click={() => handleShowDiff(version)}
            >
              View Changes
            </button>
            <button
              class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm"
              on:click={() => configStore.restoreVersion(version)}
            >
              Restore
            </button>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>