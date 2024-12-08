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
}

function createPageStore() {
  const { subscribe, set, update } = writable<PageState>({
    title: '',
    actions: []
  });

  return {
    subscribe,
    setTitle: (title: string) => update((state) => ({ ...state, title })),
    setActions: (actions: PageAction[]) => update((state) => ({ ...state, actions })),
    reset: () => set({ title: '', actions: [] })
  };
}

export const pageStore = createPageStore();
