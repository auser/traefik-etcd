<script lang="ts">
  import type { PathConfig } from '../types';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();
  export let config: PathConfig;
</script>

<div class="p-4 border rounded">
  <div class="flex gap-4 mb-4">
      <input
          type="text"
          bind:value={config.path}
          placeholder="Path"
          class="flex-1 p-2 border rounded"
      />
      <button
          on:click={() => dispatch('remove')}
          class="bg-red-500 text-white px-4 py-2 rounded"
      >
          Remove
      </button>
  </div>

  <div class="flex gap-4">
      <label class="flex items-center">
          <input
              type="checkbox"
              bind:checked={config.stripPrefix}
              class="mr-2"
          />
          Strip Prefix
      </label>
      <label class="flex items-center">
          <input
              type="checkbox"
              bind:checked={config.passThrough}
              class="mr-2"
          />
          Pass Through
      </label>
  </div>
</div>