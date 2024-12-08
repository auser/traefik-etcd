<!-- src/lib/components/config/HostCard.svelte -->
<script lang="ts">
  import type { HostConfig } from '$lib/types';
	import clsx from 'clsx';
  import { ChevronDown, Trash2 } from 'lucide-svelte';
  import { slide } from 'svelte/transition';
  
  export let host: HostConfig;
  export let onChange: (host: HostConfig) => void;
  export let onDelete: () => void;

  let expanded = false;
</script>

<div class="border rounded-lg bg-white overflow-hidden">
  <div class="p-4 flex items-center justify-between">
    <div class="flex-1">
      <input
        type="text"
        class="w-full text-lg font-medium bg-transparent border-0 focus:ring-2 focus:ring-blue-500 rounded"
        placeholder="Enter domain"
        value={host.domain}
        on:input={(e) => onChange({ ...host, domain: e.currentTarget.value })}
      />
    </div>
    
    <div class="flex items-center gap-2">
      <button
        class="p-2 text-gray-500 hover:text-gray-700 rounded-full"
        on:click={() => expanded = !expanded}
      >
        <ChevronDown
          class={clsx("w-5 h-5 transform transition-transform", expanded && "rotate-180")}
        />
      </button>
      <button
        class="p-2 text-gray-500 hover:text-red-500 rounded-full"
        on:click={onDelete}
      >
        <Trash2 class="w-5 h-5" />
      </button>
    </div>
  </div>

  {#if expanded}
    <div class="border-t p-4" transition:slide>
      <!-- Host details like paths, deployments, etc -->
      <div class="space-y-4">
        <!-- Add your detailed host configuration here -->
      </div>
    </div>
  {/if}
</div>