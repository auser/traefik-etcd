import type { Component } from 'svelte';
import { writable } from 'svelte/store';

export type PageAction = {
  label: string;
  onClick: () => void;
  icon?: Component;
};

interface PageState {
  title: string;
  actions: PageAction[];
  lastSavedMessage?: string;
}

function createPageStore() {
  const { subscribe, set, update } = writable<PageState>({
    title: '',
    actions: [],
    lastSavedMessage: undefined
  });

  return {
    subscribe,
    setTitle: (title: string) => update((state) => ({ ...state, title })),
    setActions: (actions: PageAction[]) => update((state) => ({ ...state, actions })),
    setLastSavedMessage: (lastSavedMessage: string) =>
      update((state) => ({ ...state, lastSavedMessage })),
    reset: () => set({ title: '', actions: [], lastSavedMessage: undefined })
  };
}

export const pageStore = createPageStore();
