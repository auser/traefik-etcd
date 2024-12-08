<!-- src/lib/components/config/HostSection.svelte -->
<script lang="ts">
  import type { HostConfig } from '$lib/types';
  import { Plus } from 'lucide-svelte';
  import HostCard from './HostCard.svelte';
  
  export let hosts: HostConfig[];
  export let onChange: (hosts: HostConfig[]) => void;

  function addHost() {
    onChange([
      ...hosts,
      {
        domain: '',
        paths: [],
        deployments: {},
        middlewares: [],
        selection: null
      }
    ]);
  }
</script>

<section class="bg-white rounded-lg border p-6">
  <header class="flex justify-between items-center mb-6">
    <h2 class="text-lg font-semibold">Host Configuration</h2>
    <button
      class="flex items-center gap-2 px-3 py-2 text-sm bg-blue-500 text-white rounded-md hover:bg-blue-600"
      on:click={addHost}
    >
      <Plus class="w-4 h-4" />
      Add Host
    </button>
  </header>

  {#if hosts.length === 0}
    <div class="text-center py-12 border-2 border-dashed rounded-lg">
      <p class="text-gray-500">No hosts configured yet.</p>
      <button
        class="mt-4 text-blue-500 hover:text-blue-600"
        on:click={addHost}
      >
        Add your first host
      </button>
    </div>
  {:else}
    <div class="space-y-4">
      {#each hosts as host, i (host.domain)}
        <HostCard
          {host}
          onChange={(updatedHost) => {
            const newHosts = [...hosts];
            newHosts[i] = updatedHost;
            onChange(newHosts);
          }}
          onDelete={() => {
            const newHosts = hosts.filter((_, index) => index !== i);
            onChange(newHosts);
          }}
        />
      {/each}
    </div>
  {/if}
</section>