// src/lib/stores/configStore.ts
import { writable } from 'svelte/store';
import { baseUrl, fetching } from '$lib/fetcher';
import type { TemplateInfo, TraefikConfigVersion } from '$lib/types';

export enum ConfigSource {
  Database = 'Database',
  File = 'File',
  Default = 'Default',
  New = 'New'
}

export interface ConfigListItem {
  id: number;
  name: string;
  source: ConfigSource;
  updated_at: string;
}

interface ConfigState {
  configs: ConfigListItem[];
  selectedId: number | null;
  editedContent: string;
  error: string;
  saveStatus: string;
  templates: TemplateInfo[];
}

const initialState: ConfigState = {
  configs: [],
  selectedId: null,
  editedContent: '',
  error: '',
  saveStatus: '',
  templates: []
};

function createConfigStore() {
  const { subscribe, set, update } = writable<ConfigState>(initialState);

  return {
    subscribe,
    setSelectedId: (id: number | null) => {
      update(state => ({ ...state, selectedId: id }));
    },

    selectConfig: async (id: number | null) => {
      if (id === null) {
        update(state => ({
          ...state,
          selectedId: null,
          editedContent: '',
          error: ''
        }));
        return;
      }

      try {
        if (id === 0) {  // New config
          const response = await fetching('/configs/default');
          const data = await response.json();
          console.log('selectConfig default', data);
          update(state => ({
            ...state,
            selectedId: 0,
            editedContent: data.config,
            error: ''
          }));
        } else {
          const response = await fetching(`/configs/${id}`);
          const data = await response.json();
          console.log('selectConfig', data);
          update(state => ({
            ...state,
            selectedId: id,
            editedContent: data.config,
            error: ''
          }));
        }
      } catch (err) {
        update(state => ({
          ...state,
          error: 'Failed to load configuration'
        }));
      }
    },
    fetchConfigs: async () => {
      try {
        const response = await fetching('/configs');
        const data = await response.json();
        console.log('fetchConfigs data', data);
        update(state => ({ ...state, configs: data, error: '' }));
      } catch (err) {
        update(state => ({ ...state, error: 'Failed to load configurations' }));
      }
    },
    loadConfig: async (configId: number) => {
      try {
        const response = await fetching(`/configs/${configId}`);
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
    loadConfigs: async () => {
      try {
        const response = await fetching('/configs');
        const data = await response.json();
        console.log('loadConfigs data', data);
        update(state => ({ ...state, configs: data }));
      } catch (err) {
        update(state => ({
          ...state,
          error: 'Failed to load configurations'
        }));
      }
    },

    loadTemplates: async () => {
      try {
        const response = await fetching('/templates');
        const data = await response.json();
        console.log('loadTemplates data', data);
        update(state => ({ ...state, templates: data }));
      } catch (err) {
        update(state => ({ ...state, error: 'Failed to load templates' }));
      }
    },

    loadTemplate: async (path: string) => {
      try {
        const response = await fetching(`/templates/${encodeURIComponent(path)}`);
        const data = await response.json();
        update(state => ({ ...state, template: data }));
      } catch (err) {
        update(state => ({ ...state, error: 'Failed to load template' }));
      }
    },

    saveConfig: async () => {
      update((state: ConfigState) => {
        if (!state.selectedId) return state;

        // Save as new version
        const request = {
          name: `${state.configs.find(c => c.id === state.selectedId)?.name} v${new Date().toISOString()}`,
          config: state.editedContent
        };

        fetch(baseUrl + '/configs/version', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(request)
        })
          .then(response => {
            if (!response.ok) throw new Error('Failed to save');
            return response.json();
          })
          .then(newVersion => {
            // Add the new version to our configs list
            update(s => ({
              ...s,
              configs: [...s.configs, newVersion],
              selectedId: newVersion.id,
              saveStatus: 'Configuration saved successfully',
              error: ''
            }));
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

    updateName: async (newName: string) => {
      update(state => ({
        ...state,
        configs: state.configs.map(config =>
          config.id === state.selectedId
            ? { ...config, name: newName }
            : config
        )
      }));
    },
    updateEditedContent: (content: string) => {
      update(state => ({ ...state, editedContent: content }));
    },
    reset: () => {
      update(state => ({
        ...state,
        editedContent: ''
      }));
    },
    clearError: () => {
      update(state => ({ ...state, error: '' }));
    },
    deleteConfig: async (id: number) => {
      try {
        const response = await fetching(`/configs/${id}`, {
          method: 'DELETE'
        });

        if (!response.ok) throw new Error('Failed to delete');

        update(state => ({
          ...state,
          configs: state.configs.filter(c => c.id !== id),
          selectedId: state.selectedId === id ? null : state.selectedId
        }));
      } catch (err) {
        update(state => ({ ...state, error: 'Failed to delete configuration' }));
      }
    }
  };
}

export const configStore = createConfigStore();

