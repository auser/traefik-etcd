<!-- src/lib/components/config/DiffViewer.svelte -->
<script lang="ts">
  import { diffLines } from 'diff';
  import { configStore } from '$lib/stores/configStore';
  import { stringifyYaml } from '$lib/utils/parsing';

  export let oldVersion: any;
  export let newVersion: any;

  $: diff = diffLines(
    stringifyYaml(oldVersion.config), 
    stringifyYaml(newVersion.config)
  );
</script>

<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4">
  <div class="bg-white rounded-lg w-full max-w-4xl max-h-[90vh] flex flex-col">
    <div class="p-4 border-b flex items-center justify-between">
      <h3 class="font-semibold">
        Changes between Version {oldVersion.version} and {newVersion.version}
      </h3>
      <button
        class="text-gray-500 hover:text-gray-700"
        on:click={() => configStore.closeDiff()}
      >
        ✕
      </button>
    </div>
    <div class="overflow-auto p-4 font-mono text-sm">
      {#each diff as part}
        <div
          class="whitespace-pre"
          class:bg-red-100={part.removed}
          class:bg-green-100={part.added}
        >
          {part.value}
        </div>
      {/each}
    </div>
  </div>
</div>