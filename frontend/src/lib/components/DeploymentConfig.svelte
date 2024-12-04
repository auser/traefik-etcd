<script lang="ts">
  import type { DeploymentConfig, DeploymentProtocol } from '../../types';

  export let name: string;
  export let config: DeploymentConfig;

  const protocols: DeploymentProtocol[] = ['Http', 'Https', 'Tcp'];
</script>

<div class="p-4 border rounded mb-4">
  <div class="grid grid-cols-2 gap-4">
      <input
          type="text"
          bind:value={config.name}
          placeholder="Name"
          class="p-2 border rounded"
      />
      <input
          type="text"
          bind:value={config.ip}
          placeholder="IP Address"
          class="p-2 border rounded"
      />
      <input
          type="number"
          bind:value={config.port}
          placeholder="Port"
          class="p-2 border rounded"
      />
      <input
          type="number"
          bind:value={config.weight}
          placeholder="Weight"
          class="p-2 border rounded"
      />
      <select
          bind:value={config.protocol}
          class="p-2 border rounded"
      >
          {#each protocols as protocol}
              <option value={protocol}>{protocol}</option>
          {/each}
      </select>
  </div>
</div>