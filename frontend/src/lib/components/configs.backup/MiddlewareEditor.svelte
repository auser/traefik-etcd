<!-- src/lib/components/config/MiddlewareEditor.svelte -->
<script lang="ts">
  import type { MiddlewareConfig, HeadersConfig } from '$lib/types';
  import { Plus, Trash2 } from 'lucide-svelte';

  export let name: string;
  export let middleware: MiddlewareConfig;
  export let onChange: (name: string, middleware: MiddlewareConfig) => void;

  function addRequestHeader() {
    if (!middleware.headers) {
      middleware.headers = {
        additional_headers: {},
        custom_request_headers: {},
        custom_response_headers: {},
        access_control_allow_methods: [],
        access_control_allow_headers: [],
        access_control_expose_headers: [],
        access_control_allow_origin_list: [],
        add_vary_header: false
      };
    }
    middleware.headers.custom_request_headers['New-Header'] = '';
    middleware = { ...middleware };
    onChange(name, middleware);
  }

  function addResponseHeader() {
    if (!middleware.headers) {
      middleware.headers = {
        custom_request_headers: {},
        custom_response_headers: {},
        access_control_allow_methods: [],
        access_control_allow_headers: [],
        access_control_expose_headers: [],
        access_control_allow_origin_list: [],
        add_vary_header: false
      };
    }
    middleware.headers.custom_response_headers['New-Header'] = '';
    middleware = { ...middleware };
    onChange(name, middleware);
  }
</script>

<div class="space-y-4">
  <!-- Name -->
  <div class="space-y-1">
    <label class="text-sm font-medium">Name</label>
    <input
      type="text"
      class="w-full p-2 border rounded"
      bind:value={name}
      on:change={() => onChange(name, middleware)}
      placeholder="headers-middleware"
    />
  </div>

  <!-- Protocol -->
  <div class="space-y-1">
    <label class="text-sm font-medium">Protocol</label>
    <select
      class="w-full p-2 border rounded"
      bind:value={middleware.protocol}
      on:change={() => onChange(name, middleware)}
    >
      <option value="http">HTTP</option>
      <option value="https">HTTPS</option>
      <option value="tcp">TCP</option>
    </select>
  </div>

  <!-- Headers Section -->
  {#if middleware.headers}
    <div class="space-y-4">
      <!-- Request Headers -->
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <label class="text-sm font-medium">Request Headers</label>
          <button
            class="flex items-center gap-2 px-2 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
            on:click={addRequestHeader}
          >
            <Plus class="w-3 h-3" />
            Add Header
          </button>
        </div>

        {#each Object.entries(middleware.headers.custom_request_headers) as [headerName, headerValue]}
          <div class="flex gap-2 items-center">
            <input
              type="text"
              class="flex-1 p-2 border rounded"
              bind:value={headerName}
              placeholder="Header Name"
              on:change={() => onChange(name, middleware)}
            />
            <input
              type="text"
              class="flex-1 p-2 border rounded"
              bind:value={middleware.headers.custom_request_headers[headerName]}
              placeholder="Header Value"
              on:change={() => onChange(name, middleware)}
            />
            <button
              class="text-red-500 hover:text-red-700"
              on:click={() => {
                delete middleware.headers.custom_request_headers[headerName];
                middleware = { ...middleware };
                onChange(name, middleware);
              }}
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        {/each}
      </div>

      <!-- Response Headers -->
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <label class="text-sm font-medium">Response Headers</label>
          <button
            class="flex items-center gap-2 px-2 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
            on:click={addResponseHeader}
          >
            <Plus class="w-3 h-3" />
            Add Header
          </button>
        </div>

        {#each Object.entries(middleware.headers.custom_response_headers) as [headerName, headerValue]}
          <div class="flex gap-2 items-center">
            <input
              type="text"
              class="flex-1 p-2 border rounded"
              bind:value={headerName}
              placeholder="Header Name"
              on:change={() => onChange(name, middleware)}
            />
            <input
              type="text"
              class="flex-1 p-2 border rounded"
              bind:value={middleware.headers.custom_response_headers[headerName]}
              placeholder="Header Value"
              on:change={() => onChange(name, middleware)}
            />
            <button
              class="text-red-500 hover:text-red-700"
              on:click={() => {
                delete middleware.headers.custom_response_headers[headerName];
                middleware = { ...middleware };
                onChange(name, middleware);
              }}
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        {/each}
      </div>

      <!-- CORS Settings -->
      <div class="space-y-2">
        <label class="text-sm font-medium">CORS Settings</label>
        
        <div class="space-y-2">
          <input
            type="text"
            class="w-full p-2 border rounded"
            placeholder="Allow Methods (comma-separated)"
            value={middleware.headers.access_control_allow_methods.join(', ')}
            on:change={(e) => {
              middleware.headers.access_control_allow_methods = e.currentTarget.value
                .split(',')
                .map(m => m.trim())
                .filter(Boolean);
              onChange(name, middleware);
            }}
          />

          <input
            type="text"
            class="w-full p-2 border rounded"
            placeholder="Allow Headers (comma-separated)"
            value={middleware.headers.access_control_allow_headers.join(', ')}
            on:change={(e) => {
              middleware.headers.access_control_allow_headers = e.currentTarget.value
                .split(',')
                .map(h => h.trim())
                .filter(Boolean);
              onChange(name, middleware);
            }}
          />

          <input
            type="text"
            class="w-full p-2 border rounded"
            placeholder="Allow Origins (comma-separated)"
            value={middleware.headers.access_control_allow_origin_list.join(', ')}
            on:change={(e) => {
              middleware.headers.access_control_allow_origin_list = e.currentTarget.value