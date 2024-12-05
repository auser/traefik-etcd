<!-- src/lib/components/config/VisualConfigEditor.svelte -->
<script lang="ts">
  import type { TraefikConfig, HostConfig } from '$lib/types';
  import { Plus, Trash2 } from 'lucide-svelte';
  import HostEditor from './HostEditor.svelte';

  export let config: TraefikConfig;
  export let onChange: (config: TraefikConfig) => void;
  export let onNameChange: (newName: string) => void;

  function addHost() {
    const newConfig = {
      ...config,
      hosts: [
        ...config.hosts,
        {
          domain: '',
          paths: [],
          deployments: {},
          middlewares: [],
          selection: null
        }
      ]
    };
    onChange(newConfig);
  }

  function removeHost(index: number) {
    const newHosts = [...config.hosts];
    newHosts.splice(index, 1);
    onChange({ ...config, hosts: newHosts });
  }

  function updateHost(index: number, updatedHost: HostConfig) {
    const newHosts = [...config.hosts];
    newHosts[index] = updatedHost;
    onChange({ ...config, hosts: newHosts });
  }
  
</script>

  <!-- Configuration Name -->
  <div class="space-y-2">
    <label class="text-sm font-medium">Configuration Name</label>
    <input
      type="text"
      class="w-full p-2 border rounded"
      value={name}
      on:input={(e) => onNameChange(e.currentTarget.value)}
      placeholder="My Configuration"
    />
  </div>
  
<div class="space-y-6">
  <!-- Hosts Section -->
  <div class="space-y-4">
    <div class="flex justify-between items-center">
      <h3 class="text-lg font-semibold">Hosts</h3>
      <button
        class="flex items-center gap-2 px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
        on:click={addHost}
      >
        <Plus class="w-4 h-4" />
        Add Host
      </button>
    </div>

    {#if config.hosts.length === 0}
      <div class="text-center py-8 text-gray-500 border-2 border-dashed rounded">
        No hosts configured. Click "Add Host" to begin.
      </div>
    {/if}

    {#each config.hosts as host, i (i)}
      <div class="border rounded p-4 space-y-4">
        <div class="flex justify-between items-start">
          <div class="flex-grow">
            <HostEditor 
              {host} 
              onChange={(updatedHost) => updateHost(i, updatedHost)}
            />
          </div>
          <button
            class="text-red-500 hover:text-red-700 ml-4"
            on:click={() => removeHost(i)}
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>