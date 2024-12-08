<!-- src/lib/components/config/HostsConfig.svelte -->
<script lang="ts">
  import { Plus } from 'lucide-svelte';
  import HostCard from '$lib/components/config/HostCard.svelte';
  
  export let hosts: Array<{
    domain: string;
    pass_through?: boolean;
    www_redirect?: boolean;
    forward_host?: boolean;
    paths?: Array<any>;
    deployments?: Record<string, any>;
    middlewares?: string[];
  }>;
  export let onChange: (hosts: any[]) => void;

  function addHost() {
    onChange([...hosts, {
      domain: '',
      paths: [],
      deployments: {},
      middlewares: []
    }]);
  }

  function updateHost(index: number, updatedHost: any) {
    const newHosts = [...hosts];
    newHosts[index] = updatedHost;
    onChange(newHosts);
  }

  function removeHost(index: number) {
    const newHosts = [...hosts];
    newHosts.splice(index, 1);
    onChange(newHosts);
  }
</script>

<div class="bg-white rounded-lg border p-6">
  <div class="flex justify-between items-center mb-6">
    <h2 class="text-lg font-semibold">Hosts Configuration</h2>
    <button 
      class="flex items-center gap-2 px-3 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
      on:click={addHost}
    >
      <Plus class="w-4 h-4" />
      Add Host
    </button>
  </div>

  <div class="space-y-6">
    {#if hosts.length === 0}
      <div class="text-center py-8 text-gray-500 border-2 border-dashed rounded">
        No hosts configured. Click "Add Host" to begin.
      </div>
    {:else}
      {#each hosts as host, i}
        <HostCard
          {host}
          onChange={(updatedHost: any) => updateHost(i, updatedHost)}
          onDelete={() => removeHost(i)}
        />
      {/each}
    {/if}
  </div>
</div>