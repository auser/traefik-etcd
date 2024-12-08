import { writable } from 'svelte/store';

interface TraefikConfig {
  id?: number;
  name: string;
  content: {
    http?: {
      middlewares?: Record<string, any>;
      services?: Record<string, any>;
    };
    tcp?: Record<string, any>;
    tls?: Record<string, any>;
  };
  created_at?: string;
  updated_at?: string;
}

interface ConfigState {
  configs: TraefikConfig[];
  currentConfig: TraefikConfig | null;
  isDirty: boolean;
  lastSaved: Date | null;
}

function createConfigStore() {
  const { subscribe, set, update } = writable<ConfigState>({
    configs: [],
    currentConfig: null,
    isDirty: false,
    lastSaved: null
  });

  const AUTOSAVE_DELAY = 3000; // 3 seconds
  let autosaveTimer: NodeJS.Timeout;

  return {
    subscribe,

    // Load configs from backend
    async loadConfigs() {
      try {
        const response = await fetch('/api/configs');
        const configs = await response.json();
        update(state => ({ ...state, configs }));
      } catch (error) {
        console.error('Failed to load configs:', error);
      }
    },

    // Set current config
    setCurrentConfig(config: TraefikConfig) {
      update(state => ({ ...state, currentConfig: config, isDirty: false }));
    },

    // Update current config with changes
    updateCurrentConfig(changes: Partial<TraefikConfig>) {
      update(state => {
        if (!state.currentConfig) return state;

        const updatedConfig = {
          ...state.currentConfig,
          ...changes
        };

        clearTimeout(autosaveTimer);
        autosaveTimer = setTimeout(() => {
          this.saveConfig(updatedConfig);
        }, AUTOSAVE_DELAY);

        return {
          ...state,
          currentConfig: updatedConfig,
          isDirty: true
        };
      });
    },

    // Save config to backend
    async saveConfig(config: TraefikConfig) {
      try {
        const response = await fetch('/api/configs', {
          method: config.id ? 'PUT' : 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(config)
        });

        const savedConfig = await response.json();

        update(state => ({
          ...state,
          configs: state.configs.map(c =>
            c.id === savedConfig.id ? savedConfig : c
          ),
          currentConfig: savedConfig,
          isDirty: false,
          lastSaved: new Date()
        }));
      } catch (error) {
        console.error('Failed to save config:', error);
      }
    },

    // Delete config
    async deleteConfig(id: number) {
      try {
        await fetch(`/api/configs/${id}`, { method: 'DELETE' });

        update(state => ({
          ...state,
          configs: state.configs.filter(c => c.id !== id),
          currentConfig: state.currentConfig?.id === id ? null : state.currentConfig
        }));
      } catch (error) {
        console.error('Failed to delete config:', error);
      }
    }
  };
}

export const configStore = createConfigStore();