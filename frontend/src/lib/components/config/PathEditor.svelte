<!-- src/lib/components/config/PathEditor.svelte -->
<script lang="ts">
  import type { PathConfig, DeploymentConfig, DeploymentProtocol } from '$lib/types';
  import { Plus, Trash2 } from 'lucide-svelte';
  import DeploymentEditor from './DeploymentEditor.svelte';

  export let path: PathConfig;
  export let onChange: (path: PathConfig) => void;

  function addDeployment() {
    const deploymentName = `deployment-${Object.keys(path.deployments).length + 1}`;
    path.deployments[deploymentName] = {
      name: deploymentName,
      ip: '127.0.0.1',
      port: 80,
      weight: 100,
      protocol: 'http',
      middlewares: []
    };
    path = { ...path };
    onChange(path);
  }

  function removeDeployment(name: string) {
    delete path.deployments[name];
    path = { ...path };
    onChange(path);
  }
</script>

<div class="w-full space-y-4">
  <!-- Path URL -->
  <div class="space-y-2">
    <label class="text-sm font-medium">Path</label>
    <input
      type="text"
      class="w-full p-2 border rounded"
      bind:value={path.path}
      on:change={() => onChange(path)}
      placeholder="/api"
    />
  </div>

  <!-- Path Options -->
  <div class="flex gap-4">
    <label class="flex items-center gap-2">
      <input
        type="checkbox"
        class="form-checkbox"
        bind:checked={path.strip_prefix}
        on:change={() => onChange(path)}
      />
      <span class="text-sm">Strip Prefix</span>
    </label>

    <label class="flex items-center gap-2">
      <input
        type="checkbox"
        class="form-checkbox"
        bind:checked={path.pass_through}
        on:change={() => onChange(path)}
      />
      <span class="text-sm">Pass Through</span>
    </label>
  </div>

  <!-- Path Deployments -->
  <div class="space-y-2">
    <div class="flex justify-between items-center">
      <label class="text-sm font-medium">Path Deployments</label>
      <button
        class="flex items-center gap-2 px-2 py-1 text-sm bg-purple-500 text-white rounded hover:bg-purple-600"
        on:click={addDeployment}
      >
        <Plus class="w-3 h-3" />
        Add Deployment
      </button>
    </div>

    {#each Object.entries(path.deployments) as [name, deployment]}
      <div class="border rounded p-3">
        <div class="flex justify-between">
          <DeploymentEditor
            {name}
            {deployment}
            onChange={(updatedName, updatedDeployment) => {
              if (name !== updatedName) {
                delete path.deployments[name];
              }
              path.deployments[updatedName] = updatedDeployment;
              path = { ...path };
              onChange(path);
            }}
          />
          <button
            class="text-red-500 hover:text-red-700"
            on:click={() => removeDeployment(name)}
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>