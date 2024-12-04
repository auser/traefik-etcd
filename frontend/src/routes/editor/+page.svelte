<script lang="ts">
    'use client';
    import { onMount } from 'svelte';
    // import TraefikConfig from '@/components/TraefikConfig.svelte';
    import type { TraefikConfig as TraefikConfigType } from '$lib/types';
    // import { useTraefikStore } from '@/stores/traefikConfigStore'



    let configs: Array<{
        id: number;
        name: string;
        config: TraefikConfigType;
        created_at: string;
    }> = [];

    let currentConfig: TraefikConfigType = {
        rulePrefix: '',
        hosts: [],
        middlewares: {},
        etcd: {},
    };

    let configName = '';

    onMount(async () => {
        const response = await fetch('http://localhost:3000/api/configs');
        console.log(response);
        configs = await response.json();
    });

    async function saveConfig() {
        if (!configName) return;
        
        const response = await fetch('http://localhost:3000/api/configs', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                name: configName,
                config: currentConfig,
            }),
        });
        
        const newConfig = await response.json();
        configs = [newConfig, ...configs];
        configName = '';
    }

    function loadConfig(config: TraefikConfigType) {
        currentConfig = { ...config };
    }

    function exportConfig(config: TraefikConfigType) {
        const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'traefik-config.json';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
</script>

<div class="container mx-auto p-4">
    <h1 class="text-3xl font-bold mb-6">Traefik Configuration Manager</h1>

    <div class="mb-8">
        <div class="flex gap-4 mb-4">
            <input
                type="text"
                bind:value={configName}
                placeholder="Configuration Name"
                class="border p-2 rounded"
            />
            <button
                on:click={saveConfig}
                class="bg-blue-500 text-white px-4 py-2 rounded"
                disabled={!configName}
            >
                Save Configuration
            </button>
        </div>

        <!-- <TraefikConfig bind:config={currentConfig} /> -->
      </div>

      <div>
          <h2 class="text-xl font-semibold mb-4">Saved Configurations</h2>
          <div class="grid gap-4">
              {#each configs as config}
                  <div class="border p-4 rounded">
                      <div class="flex justify-between items-center mb-2">
                          <h3 class="font-semibold">{config.name}</h3>
                          <div class="flex gap-2">
                              <button
                                  on:click={() => loadConfig(config.config)}
                                  class="bg-blue-500 text-white px-3 py-1 rounded"
                              >
                                  Load
                              </button>
                              <button
                                  on:click={() => exportConfig(config.config)}
                                  class="bg-green-500 text-white px-3 py-1 rounded"
                              >
                                  Export
                              </button>
                          </div>
                      </div>
                      <div class="text-sm text-gray-600">
                          Created: {new Date(config.created_at).toLocaleString()}
                      </div>
                  </div>
              {/each}
          </div>
      </div>
  </div>