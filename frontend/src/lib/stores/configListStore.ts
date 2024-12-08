import { writable } from 'svelte/store';
import { fetching } from '$lib/utils/fetching';

export interface ConfigListItem {
  id: number;
  name: string;
  updated_at: string;
}

interface ConfigListState {
  configs: ConfigListItem[];
  loading: boolean;
  error: string | null;
}

function createConfigListStore() {
  const { subscribe, update } = writable<ConfigListState>({
    configs: [],
    loading: false,
    error: null
  });

  return {
    subscribe,

    loadConfigs: async (searchTerm: string = '') => {
      update(state => ({ ...state, loading: true }));
      try {
        const response = await fetching(`/configs/search?term=${encodeURIComponent(searchTerm)}`);
        if (!response.ok) throw new Error('Failed to load configurations');
        const configs = await response.json();
        update(state => ({ ...state, configs, loading: false }));
        return configs;
      } catch (e) {
        console.error('Failed to load configurations:', e);
        update(state => ({
          ...state,
          error: 'Failed to load configurations',
          loading: false
        }));
        return [];
      }
    },

    deleteConfig: async (id: number) => {
      update(state => ({ ...state, loading: true }));
      try {
        const response = await fetching(`/configs/${id}`, {
          method: 'DELETE'
        });

        if (!response.ok) throw new Error('Failed to delete configuration');

        // Remove the deleted config from the store
        update(state => ({
          ...state,
          configs: state.configs.filter(config => config.id !== id),
          loading: false
        }));

        return true;
      } catch (e) {
        console.error('Failed to delete configuration:', e);
        update(state => ({
          ...state,
          error: 'Failed to delete configuration',
          loading: false
        }));
        return false;
      }
    }
  };
}

export const configListStore = createConfigListStore();