// src/lib/stores/configStore.ts
import { writable } from 'svelte/store';
import type { TraefikConfig } from '$lib/types';
import { fetching } from '$lib/utils/fetching';

interface ConfigState {
  currentConfig: TraefikConfig | null;
  configName: string;
  configId: number | null;
  loading: boolean;
  error: string | null;
}

function createConfigStore() {
  const { subscribe, set, update } = writable<ConfigState>({
    currentConfig: null,
    configName: '',
    configId: null,
    loading: false,
    error: null
  });

  return {
    subscribe,

    loadConfig: async (id: number) => {
      update(s => ({ ...s, loading: true }));
      try {
        const response = await fetching(`/configs/${id}`);
        const data = await response.json();
        update(s => ({
          ...s,
          currentConfig: data.config,
          configName: data.name,
          configId: data.id,
          loading: false
        }));
      } catch (e) {
        update(s => ({ ...s, error: 'Failed to load config', loading: false }));
      }
    },

    updateConfig: (config: TraefikConfig) => {
      update(s => ({ ...s, currentConfig: config }));
    },

    updateName: (name: string) => {
      update(s => ({ ...s, configName: name }));
    },

    reset: () => {
      set({
        currentConfig: null,
        configName: '',
        configId: null,
        loading: false,
        error: null
      });
    },

    clearError: () => {
      update(s => ({ ...s, error: null }));
    }
  };
}

export const configStore = createConfigStore();