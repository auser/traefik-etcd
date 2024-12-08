// src/lib/stores/etcdStore.ts
import { writable } from 'svelte/store';

export interface TlsConfig {
  cert?: string;
  key?: string;
  ca?: string;
  domain?: string;
}

export interface EtcdConfig {
  endpoints: string[];
  timeout: number;
  keep_alive: number;
  tls?: TlsConfig;
}

const DEFAULT_CONFIG: EtcdConfig = {
  endpoints: ["http://localhost:2379"],
  timeout: 2000,
  keep_alive: 300,
};

function createEtcdStore() {
  const { subscribe, set, update } = writable<EtcdConfig>(DEFAULT_CONFIG);

  return {
    subscribe,
    set,
    update: (config: Partial<EtcdConfig>) => {
      update(current => ({ ...current, ...config }));
    },
    reset: () => set(DEFAULT_CONFIG),
    setTls: (tls: TlsConfig | undefined) => {
      update(current => ({ ...current, tls }));
    }
  };
}

export const etcdStore = createEtcdStore();