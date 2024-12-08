<!-- src/lib/components/config/HostEditor.svelte -->
<script lang="ts">
  import type { HostConfig, PathConfig } from '$lib/types';
  import { Plus, Trash2 } from 'lucide-svelte';
  import PathEditor from './PathEditor.svelte';
  import DeploymentEditor from './DeploymentEditor.svelte';

  export let host: HostConfig;
  export let onChange: (host: HostConfig) => void;

  function addPath() {
    const newHost = {
      ...host,
      paths: [
        ...host.paths,
        {
          path: '',
          deployments: {},
          middlewares: [],
          strip_prefix: false,
          pass_through: false
        }
      ]
    };
    onChange(newHost);
  }

  function removePath(index: number) {
    const newPaths = [...host.paths];
    newPaths.splice(index, 1);
    onChange({ ...host, paths: newPaths });
  }

  function updatePath(index: number, updatedPath: PathConfig) {
    const newPaths = [...host.paths];
    newPaths[index] = updatedPath;
    onChange({ ...host, paths: newPaths });
  }

  function addDeployment() {
    const deploymentName = `deployment-${Object.keys(host.deployments).length + 1}`;
    const newDeployments = {
      ...host.deployments,
      [deploymentName]: {
        name: deploymentName,
        ip: '127.0.0.1',
        port: 80,
        weight: 100,
        protocol: 'http',
        middlewares: []
      }
    };
    onChange({ ...host, deployments: newDeployments });
  }
</script>

<div class="w-full space-y-4">
  <!-- Domain -->
  <div class="space-y-2">
    <label class="text-sm font-medium">Domain</label>
    <input
      type="text"
      class="w-full p-2 border rounded"
      value={host.domain}
      on:input={(e) => onChange({ ...host, domain: e.currentTarget.value })}
      placeholder="example.com"
    />
  </div>

  <!-- Paths -->
  <div class="space-y-2">
    <div class="flex justify-between items-center">
      <label class="text-sm font-medium">Paths</label>
      <button
        class="flex items-center gap-2 px-2 py-1 text-sm bg-green-500 text-white rounded hover:bg-green-600"
        on:click={addPath}
      >
        <Plus class="w-3 h-3" />
        Add Path
      </button>
    </div>
    
    {#each host.paths as path, i (i)}
      <div class="border rounded p-3">
        <div class="flex justify-between">
          <div class="flex-grow">
            <PathEditor
              {path}
              onChange={(updatedPath) => updatePath(i, updatedPath)}
            />
          </div>
          <button
            class="text-red-500 hover:text-red-700 ml-4"
            on:click={() => removePath(i)}
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Deployments -->
  <div class="space-y-2">
    <div class="flex justify-between items-center">
      <label class="text-sm font-medium">Deployments</label>
      <button
        class="flex items-center gap-2 px-2 py-1 text-sm bg-purple-500 text-white rounded hover:bg-purple-600"
        on:click={addDeployment}
      >
        <Plus class="w-3 h-3" />
        Add Deployment
      </button>
    </div>
    
    {#each Object.entries(host.deployments) as [name, deployment] (name)}
      <div class="border rounded p-3">
        <DeploymentEditor
          {name}
          {deployment}
          onChange={(updatedName, updatedDeployment) => {
            const newDeployments = { ...host.deployments };
            if (name !== updatedName) {
              delete newDeployments[name];
            }
            newDeployments[updatedName] = updatedDeployment;
            onChange({ ...host, deployments: newDeployments });
          }}
        />
      </div>
    {/each}
  </div>
</div>