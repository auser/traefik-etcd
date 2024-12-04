// src/lib/stores/configStore.ts
import { writable } from 'svelte/store';
import { baseUrl } from '$lib/fetcher';
import type { TraefikConfigVersion } from '$lib/types';

interface ConfigStore {
  configs: any[];
  selectedConfig: any | null;
  editedContent: string;
  error: string;
  saveStatus: string;
}

const initialState: ConfigStore = {
  configs: [],
  selectedConfig: null,
  editedContent: '',
  error: '',
  saveStatus: ''
};

function createConfigStore() {
  const { subscribe, set, update } = writable<ConfigStore>(initialState);

  return {
    subscribe,
    fetchConfigs: async () => {
      try {
        const response = await fetch(baseUrl + '/configs');
        const data = await response.json();
        update(state => ({ ...state, configs: data, error: '' }));
      } catch (err) {
        update(state => ({ ...state, error: 'Failed to load configurations' }));
      }
    },
    loadConfig: async (configId: number) => {
      try {
        const response = await fetch(baseUrl + `/configs/${configId}`);
        const data = await response.json();
        update(state => ({
          ...state,
          selectedConfig: data,
          editedContent: data.config,
          error: ''
        }));
      } catch (err) {
        update(state => ({ ...state, error: 'Failed to load configuration' }));
      }
    },
    saveConfig: async () => {
      update(state => {
        if (!state.selectedConfig) return state;

        fetch(baseUrl + `/configs/${state.selectedConfig.id}`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            config: state.editedContent
          })
        })
          .then(response => {
            if (!response.ok) throw new Error('Failed to save');
            return {
              ...state,
              saveStatus: 'Configuration saved successfully',
              error: ''
            };
          })
          .catch(() => ({
            ...state,
            error: 'Failed to save configuration',
            saveStatus: ''
          }));

        setTimeout(() => {
          update(s => ({ ...s, saveStatus: '' }));
        }, 3000);

        return state;
      });
    },
    updateEditedContent: (content: string) => {
      update(state => ({ ...state, editedContent: content }));
    },
    reset: () => {
      update(state => ({
        ...state,
        editedContent: state.selectedConfig?.config || ''
      }));
    },
    clearError: () => {
      update(state => ({ ...state, error: '' }));
    }
  };
}

export const configStore = createConfigStore();