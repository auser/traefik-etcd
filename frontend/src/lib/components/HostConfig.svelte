<script lang="ts">
  import PathConfig from './PathConfig.svelte';
  import DeploymentConfig from './DeploymentConfig.svelte';
  import type { HostConfig } from '../types';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();
  export let config: HostConfig;

  function addPath() {
      config.paths = [...config.paths, {
          path: '',
          deployments: {},
          middlewares: [],
          stripPrefix: false,
          passThrough: false
      }];
  }
</script>

<div class="p-4 border rounded mb-4">
  <div class="flex justify-between mb-4">
      <input
          type="text"
          bind:value={config.domain}
          placeholder="Domain"
          class="w-1/2 p-2 border rounded"
      />
      <button
          on:click={() => dispatch('remove')}
          class="bg-red-500 text-white px-4 py-2 rounded"
      >
          Remove Host
      </button>
  </div>

  <div class="space-y-4">
      <h4 class="font-semibold">Paths</h4>
      {#each config.paths as path, i}
          <PathConfig
              bind:config={path}
              on:remove={() => config.paths.splice(i, 1)}
          />
      {/each}
      <button
          on:click={addPath}
          class="bg-blue-500 text-white px-4 py-2 rounded"
      >
          Add Path
      </button>
  </div>

  <div class="mt-4">
      <h4 class="font-semibold mb-2">Deployments</h4>
      {#each Object.entries(config.deployments) as [name, deployment]}
          <DeploymentConfig
              {name}
              bind:config={deployment}
          />
      {/each}
  </div>
</div>
