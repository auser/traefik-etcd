import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import { fetching } from '$lib/utils/fetching';
import type { TemplateInfo, TraefikConfig } from '$lib/types';
import { parseYaml, stringifyYaml } from '$lib/utils/parsing';
import { debug } from '$lib/utils/logging';


const LOCAL_STORAGE_KEY = 'traefikctl-draft';
const AUTO_SAVE_DELAY = 2000; // 2 seconds

export enum ConfigSource {
  Database = 'Database',
  File = 'File',
  Default = 'Default',
  New = 'New'
}

interface DraftConfig {
  id: number | null;
  name: string;
  config: TraefikConfig;
  templateName?: string;
  lastSaved: string;
}

export interface ConfigListItem {
  id: number;
  name: string;
  source: ConfigSource;
  updated_at: string;
}

interface ConfigState {
  currentConfig: TraefikConfig | null;
  configName: string;
  configId: number | null;
  selectedTemplate: TemplateInfo | null;
  templates: TemplateInfo[] | null;
  hasUnsavedChanges: boolean;
  loading: boolean;
  saving: boolean;
  error: string;
  saveStatus: string;
  currentVersion: number;
  versions: any[];
  showDiff: boolean;
  diffOldVersion: any;
  diffNewVersion: any;
  draft: DraftConfig | null;
  originalConfig: TraefikConfig | null;
}

function createConfigStore() {
  let autoSaveTimer: NodeJS.Timeout;

  const initialState: ConfigState = {
    currentConfig: null,
    configName: '',
    configId: null,
    selectedTemplate: null,
    templates: null,
    hasUnsavedChanges: false,
    loading: false,
    saving: false,
    error: '',
    saveStatus: '',
    currentVersion: 1,
    versions: [],
    showDiff: false,
    diffOldVersion: null,
    diffNewVersion: null,
    draft: browser ? loadDraftFromStorage() : null,
    originalConfig: null
  };

  const { subscribe, set, update } = writable<ConfigState>(initialState);

  function loadDraftFromStorage(): DraftConfig | null {
    try {
      const saved = localStorage.getItem(LOCAL_STORAGE_KEY);
      if (saved) {
        return JSON.parse(saved);
      }
    } catch (e) {
      console.error('Failed to load draft from storage:', e);
    }
    return null;
  }

  function saveDraftToStorage(draft: DraftConfig) {
    try {
      localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(draft));
    } catch (e) {
      console.error('Failed to save draft to storage:', e);
    }
  }

  function clearDraft() {
    if (browser) {
      localStorage.removeItem(LOCAL_STORAGE_KEY);
    }
    update(state => ({ ...state, draft: null }));
  }

  async function fetchVersionHistory(configId: number) {
    try {
      const response = await fetching(`/configs/history/${configId}`);
      if (!response.ok) throw new Error('Failed to fetch version history');
      const versions = await response.json();
      update(state => ({ ...state, versions }));
    } catch (e) {
      update(state => ({ ...state, error: 'Failed to load version history' }));
    }
  }

  async function createBackup(state: ConfigState) {
    if (!state.configId) return;

    try {
      await fetching(`/configs/backup/${state.configId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: state.configName,
          config: state.currentConfig
        })
      });
    } catch (e) {
      debug('Failed to create backup', e);
    }
  }

  async function saveConfig(state: ConfigState) {
    if (!state.currentConfig) return null;

    const configData = {
      name: state.configName,
      config: stringifyYaml(state.currentConfig)
    };

    if (!state.configId) {
      const response = await fetching('/configs/version', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(configData)
      });

      if (!response.ok) throw new Error('Failed to save new config');
      return await response.json();
    } else {
      const response = await fetching(`/configs/update/${state.configId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(configData)
      });

      if (!response.ok) throw new Error('Failed to update config');
      return await response.json();
    }
  }

  function autosave(state: ConfigState) {
    if (autoSaveTimer) {
      clearTimeout(autoSaveTimer);
    }

    autoSaveTimer = setTimeout(() => {
      const draft: DraftConfig = {
        id: state.configId,
        name: state.configName,
        config: state.currentConfig!,
        templateName: state.selectedTemplate?.name,
        lastSaved: new Date().toISOString()
      };

      if (browser) {
        saveDraftToStorage(draft);
      }

      debug('Autosaved draft', draft);
      update(s => ({ ...s, draft }));
    }, AUTO_SAVE_DELAY);
  }

  async function save(state: ConfigState) {
    try {
      debug('Saving config', state);
      await createBackup(state);
      debug('Saved backup');
      const result = await saveConfig(state);
      debug('Saved config', result);
      if (result) {
        debug('Fetching version history and config');
        await Promise.all([
          fetchVersionHistory(result.id),
          fetching(`/configs/id/${result.id}`).then(r => r.json())
        ]);

        update(state => ({
          ...state,
          configId: result.id,
          currentVersion: result.version,
          hasUnsavedChanges: false,
          saving: false,
          saveStatus: 'Saved successfully'
        }));
        clearDraft();
      }
    } catch (e) {
      update(state => ({
        ...state,
        saving: false,
        error: 'Failed to save configuration'
      }));
    }
  }

  function reset() {
    update(() => initialState);
  }

  function discardDraft() {
    configStore.clearDraft();
    update(state => ({ ...state, showDraftDialog: false }));
  }

  return {
    subscribe,
    set,
    update,

    updateConfig: (config: TraefikConfig) => {
      update(state => {
        const newState = {
          ...state,
          currentConfig: config,
          hasUnsavedChanges: true
        };
        autosave(newState);
        return newState;
      });
    },


    updateName: (name: string) => {
      update(state => {
        const newState = {
          ...state,
          configName: name,
          hasUnsavedChanges: true
        };
        autosave(newState);
        return newState;
      });
    },

    autosave,

    save: () => {
      update(state => ({ ...state, saving: true }));
      const currentState = get();
      save(currentState);
    },

    loadDraft: () => {
      const draft = get().draft;
      if (draft) {
        update(state => ({
          ...state,
          currentConfig: draft.config,
          configName: draft.name,
          configId: draft.id,
          selectedTemplate: draft.templateName ? { name: draft.templateName } as TemplateInfo : null,
          hasUnsavedChanges: true
        }));
      }
    },
    discardDraft,

    showDiff: (oldVersion: any, newVersion: any) => {
      update(state => ({
        ...state,
        showDiff: true,
        diffOldVersion: oldVersion,
        diffNewVersion: newVersion
      }));
    },

    closeDiff: () => {
      update(state => ({
        ...state,
        showDiff: false,
        diffOldVersion: null,
        diffNewVersion: null
      }));
    },

    reset,

    restoreVersion: async (version: any) => {
      console.log('Restoring version', version);
      update(state => ({
        ...state,
        currentConfig: version.config,
        hasUnsavedChanges: true
      }));
      update(state => ({ ...state, saving: true }));
      const currentState = get();
      save(currentState);
    },

    deleteCurrentConfig: async () => {
      const state = get();
      if (!state.configId) return;

      try {
        const response = await fetching(`/configs/delete/${state.configId}`, {
          method: 'DELETE'
        });

        if (!response.ok) throw new Error('Failed to delete');

        reset();
        clearDraft();
      } catch (e) {
        update(s => ({ ...s, error: 'Failed to delete configuration' }));
      }
    },

    clearError: () => {
      update(state => ({ ...state, error: '' }));
    },

    clearDraft,

    loadTemplates: async () => {
      const response = await fetching('/templates');
      const templates = await response.json();
      update(state => ({ ...state, templates }));
      return templates;
    },

    loadConfigs: async (searchTerm: string) => {
      const response = await fetching('/templates/search', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
        params: {
          search_term: searchTerm
        }
      });
      const configs = await response.json();
      update(state => ({ ...state, configs }));
      return configs;
    },

    deleteTemplate: async (template: TemplateInfo) => {
      console.log('Deleting template', template);
      update(state => ({ ...state, loading: true }));
      try {
        const response = await fetching(`/templates/delete/${template.id}`, {
          method: 'DELETE'
        });

        if (!response.ok) throw new Error('Failed to delete template');
        reset();
      } catch (e) {
        update(state => ({ ...state, loading: false, error: 'Failed to delete template' }));
      } finally {
        update(state => ({ ...state, loading: false }));
      }
    },

    selectTemplate: async (template: TemplateInfo) => {
      update(state => ({ ...state, loading: true }));

      try {
        // Fetch template content
        const url = template.id ?
          `/configs/id/${template.id}` :
          `/templates/name/${encodeURIComponent(template.name)}`;

        const response = await fetching(url);
        if (!response.ok) throw new Error('Failed to fetch template');
        const content = await response.json();

        const configContent = content.config ? parseYaml(content.config) : content;

        // Create new config from template
        const newConfigResponse = await fetching('/configs/version', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            name: `New config based on ${template.name}`,
            config: stringifyYaml(configContent)
          })
        });

        if (!newConfigResponse.ok) throw new Error('Failed to create new config');
        const newConfig = await newConfigResponse.json();

        update(state => ({
          ...state,
          currentConfig: configContent,
          originalConfig: structuredClone(configContent),
          selectedTemplate: template,
          configName: newConfig.name,
          configId: newConfig.id,
          currentVersion: content.version || 1,
          hasUnsavedChanges: false,
          loading: false,
          error: ''
        }));

        // Save draft
        const draft: DraftConfig = {
          id: newConfig.id,
          name: newConfig.name,
          config: configContent,
          templateName: template.name,
          lastSaved: new Date().toISOString()
        };

        if (browser) {
          saveDraftToStorage(draft);
        }

      } catch (e) {
        update(state => ({
          ...state,
          loading: false,
          error: 'Failed to load template'
        }));
      }
    }
  };
}

export const configStore = createConfigStore();

// Helper to get current state
function get() {
  let state: ConfigState;
  configStore.subscribe(s => state = s)();
  return state!;
}