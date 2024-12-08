// src/lib/stores/draftStore.ts
import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import type { TraefikConfig } from '$lib/types';

const DRAFT_KEY = 'traefikctl-draft';

interface Draft {
  id: number | null;
  name: string;
  config: TraefikConfig;
  lastSaved: string;
}

function createDraftStore() {
  const { subscribe, set, update } = writable<Draft | null>(
    browser ? loadDraftFromStorage() : null
  );

  function loadDraftFromStorage(): Draft | null {
    try {
      const saved = localStorage.getItem(DRAFT_KEY);
      return saved ? JSON.parse(saved) : null;
    } catch (e) {
      console.error('Failed to load draft:', e);
      return null;
    }
  }

  function saveDraftToStorage(draft: Draft) {
    try {
      localStorage.setItem(DRAFT_KEY, JSON.stringify(draft));
    } catch (e) {
      console.error('Failed to save draft:', e);
    }
  }

  return {
    subscribe,

    saveDraft: (config: TraefikConfig, name: string, id: number | null) => {
      const draft = {
        id,
        name,
        config,
        lastSaved: new Date().toISOString()
      };

      if (browser) {
        saveDraftToStorage(draft);
      }

      set(draft);
    },

    clearDraft: () => {
      if (browser) {
        localStorage.removeItem(DRAFT_KEY);
      }
      set(null);
    }
  };
}

export const draftStore = createDraftStore();