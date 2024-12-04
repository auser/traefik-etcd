import { writable } from 'svelte/store';
import type { TraefikctlConfigHostPathConfigSchema, TraefikctlConfigMiddlewareMiddlewareConfigSchema, TraefikctlConfigTraefikConfigTraefikConfigSchema } from '$lib/types/traefik';
import { useWritable } from './use-shared-store';

const initialTraefikConfig: TraefikctlConfigTraefikConfigTraefikConfigSchema = {
  rulePrefix: '',
  hosts: [],
  middlewares: {},
};

export const traefikConfigStore = () => {
  const { set, update, subscribe } = useWritable<TraefikctlConfigTraefikConfigTraefikConfigSchema>('traefikConfigStore', initialTraefikConfig);
  return {
    set,
    update,
    subscribe,
    addHost: (host: TraefikctlConfigHostPathConfigSchema) => {
      update((state) => ({
        ...state,
        hosts: [...state.hosts, host],
      }));
    },
    addMiddleware: (name: string, middleware: TraefikctlConfigMiddlewareMiddlewareConfigSchema) => {
      update((state) => ({
        ...state,
        middlewares: { ...state.middlewares, [name]: middleware },
      }));
    },
  }
}


// import type DeploymentConfig from '$lib/components/DeploymentConfig.svelte';
// import type MiddlewareConfig from '$lib/components/MiddlewareConfig.svelte';
// import type PathConfig from '$lib/components/PathConfig.svelte';
// import type { TraefikctlConfigSelectionsSelectionConfigSchema } from '$lib/types/traefik';
// import { create } from 'zustand'
// import { immer } from 'zustand/middleware/immer'

// interface TraefikConfig {
//   rulePrefix: string;
//   hosts: HostConfig[];
//   middlewares: Record<string, MiddlewareConfig>;
// }

// interface HostConfig {
//   domain: string;
//   paths: PathConfig[];
//   deployments: Record<string, DeploymentConfig>;
//   middlewares: string[];
//   selection?: TraefikctlConfigSelectionsSelectionConfigSchema;
// }

// interface ConfigVersion {
//   id: number;
//   name: string;
//   config: TraefikConfig;
//   created_at: string;
//   updated_at: string;
//   version: number;
// }

// interface TraefikStore {
//   // Current working config
//   currentConfig: TraefikConfig | null;
//   // Saved configs from backend
//   savedConfigs: ConfigVersion[];
//   // Actions
//   setCurrentConfig: (config: TraefikConfig) => void;
//   updateHost: (index: number, host: HostConfig) => void;
//   addHost: (host: HostConfig) => void;
//   removeHost: (index: number) => void;
//   addMiddleware: (name: string, middleware: MiddlewareConfig) => void;
//   removeMiddleware: (name: string) => void;
//   // Backend interactions
//   saveConfig: (name: string) => Promise<void>;
//   loadConfigs: () => Promise<void>;
// }

// export const useTraefikStore = create<TraefikStore>()(
//   immer((set, get) => ({
//     currentConfig: null,
//     savedConfigs: [],

//     setCurrentConfig: (config) => {
//       set((state) => {
//         state.currentConfig = config
//       })
//     },

//     updateHost: (index, host) => {
//       set((state) => {
//         if (state.currentConfig) {
//           state.currentConfig.hosts[index] = host
//         }
//       })
//     },

//     addHost: (host) => {
//       set((state) => {
//         if (state.currentConfig) {
//           state.currentConfig.hosts.push(host)
//         }
//       })
//     },

//     removeHost: (index) => {
//       set((state) => {
//         if (state.currentConfig) {
//           state.currentConfig.hosts.splice(index, 1)
//         }
//       })
//     },

//     addMiddleware: (name, middleware) => {
//       set((state) => {
//         if (state.currentConfig) {
//           state.currentConfig.middlewares[name] = middleware
//         }
//       })
//     },

//     removeMiddleware: (name) => {
//       set((state) => {
//         if (state.currentConfig) {
//           delete state.currentConfig.middlewares[name]
//         }
//       })
//     },

//     saveConfig: async (name) => {
//       const state = get()
//       if (!state.currentConfig) return

//       try {
//         const response = await fetch('http://localhost:3000/api/configs', {
//           method: 'POST',
//           headers: {
//             'Content-Type': 'application/json',
//           },
//           body: JSON.stringify({
//             name,
//             config: state.currentConfig,
//           }),
//         })

//         if (!response.ok) {
//           throw new Error('Failed to save config')
//         }

//         const savedConfig = await response.json()
//         set((state) => {
//           state.savedConfigs.unshift(savedConfig)
//         })
//       } catch (error) {
//         console.error('Failed to save config:', error)
//         throw error
//       }
//     },

//     loadConfigs: async () => {
//       try {
//         const response = await fetch('http://localhost:3000/api/configs')
//         if (!response.ok) {
//           throw new Error('Failed to load configs')
//         }

//         const configs = await response.json()
//         set((state) => {
//           state.savedConfigs = configs
//         })
//       } catch (error) {
//         console.error('Failed to load configs:', error)
//         throw error
//       }
//     },
//   }))
// )