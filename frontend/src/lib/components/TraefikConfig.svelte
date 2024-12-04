<script lang="ts">
  import HostConfig from './HostConfig.svelte';
  import MiddlewareConfig from './MiddlewareConfig.svelte';
  import type { TraefikConfig } from '../types';

  export let config: TraefikConfig;

  function addHost() {
      config.hosts = [...config.hosts, {
          domain: '',
          paths: [],
          deployments: {},
          middlewares: []
      }];
  }
</script>

<div class="space-y-4">
  <div class="p-4 border rounded">
      <label class="block mb-2">
          Rule Prefix
          <input
              type="text"
              bind:value={config.rulePrefix}
              class="w-full p-2 border rounded"
          />
      </label>
  </div>

  <div class="p-4 border rounded">
      <h3 class="text-lg font-semibold mb-4">Hosts</h3>
      {#each config.hosts as host, i}
          <HostConfig bind:config={host} on:remove={() => config.hosts.splice(i, 1)} />
      {/each}
      <button
          on:click={addHost}
          class="mt-4 bg-green-500 text-white px-4 py-2 rounded"
      >
          Add Host
      </button>
  </div>

  <div class="p-4 border rounded">
      <h3 class="text-lg font-semibold mb-4">Middlewares</h3>
      {#each Object.entries(config.middlewares) as [name, middleware]}
          <MiddlewareConfig {name} bind:config={middleware} />
      {/each}
  </div>
</div>
