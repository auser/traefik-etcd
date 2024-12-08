<!-- src/lib/components/config/DeploymentEditor.svelte -->
<script lang="ts">
  import type { DeploymentConfig } from '$lib/types';
  import { DeploymentProtocol } from '$lib/types';
  export let name: string;
  export let deployment: DeploymentConfig;
  export let onChange: (name: string, deployment: DeploymentConfig) => void;

  const protocols = [
    { value: DeploymentProtocol.Http, label: 'HTTP' },
    { value: DeploymentProtocol.Https, label: 'HTTPS' },
    { value: DeploymentProtocol.Tcp, label: 'TCP' }
  ];

  function validatePort(port: number): boolean {
    return port > 0 && port < 65536;
  }

  function validateWeight(weight: number): boolean {
    return weight >= 0 && weight <= 100;
  }
</script>

<div class="grid grid-cols-2 gap-4">
  <!-- Name -->
  <div class="space-y-1">
    <label class="text-sm font-medium">Name</label>
    <input
      type="text"
      class="w-full p-2 border rounded"
      bind:value={name}
      on:change={() => onChange(name, deployment)}
      placeholder="blue"
    />
  </div>

  <!-- Protocol -->
  <div class="space-y-1">
    <label class="text-sm font-medium">Protocol</label>
    <select
      class="w-full p-2 border rounded"
      bind:value={deployment.protocol}
      on:change={() => onChange(name, deployment)}
    >
      {#each protocols as protocol}
        <option value={protocol.value}>{protocol.label}</option>
      {/each}
    </select>
  </div>

  <!-- IP -->
  <div class="space-y-1">
    <label class="text-sm font-medium">IP/Hostname</label>
    <input
      type="text"
      class="w-full p-2 border rounded"
      bind:value={deployment.ip}
      on:change={() => onChange(name, deployment)}
      placeholder="127.0.0.1"
    />
  </div>

  <!-- Port -->
  <div class="space-y-1">
    <label class="text-sm font-medium">Port</label>
    <input
      type="number"
      class="w-full p-2 border rounded"
      bind:value={deployment.port}
      on:change={() => {
        if (validatePort(deployment.port)) {
          onChange(name, deployment);
        }
      }}
      min="1"
      max="65535"
    />
  </div>

  <!-- Weight -->
  <div class="space-y-1">
    <label class="text-sm font-medium">Weight</label>
    <input
      type="number"
      class="w-full p-2 border rounded"
      bind:value={deployment.weight}
      on:change={() => {
        if (validateWeight(deployment.weight)) {
          onChange(name, deployment);
        }
      }}
      min="0"
      max="100"
    />
  </div>
</div>