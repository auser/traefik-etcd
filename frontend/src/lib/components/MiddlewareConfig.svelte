<script lang="ts">
  import type { MiddlewareConfig } from '../types';
  import HeadersConfig from './HeadersConfig.svelte';

  export let name: string;
  export let config: MiddlewareConfig;
</script>

<div class="p-4 border rounded mb-4">
  <input
      type="text"
      bind:value={config.name}
      placeholder="Name"
      class="w-full p-2 border rounded mb-4"
  />
  
  {#if config.headers}
      <HeadersConfig bind:config={config.headers} />
  {/if}
</div>