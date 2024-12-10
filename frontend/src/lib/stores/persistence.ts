// src/lib/stores/persistence.ts
import { browser } from '$app/environment';
import { configStore } from './configStore';
import { pageStore } from './pageStore';

const STORAGE_KEY = 'traefikctl-state';
const CURRENT_VERSION = 1;


// Define the structure of our entire application state
interface AppState {
  config: any;  // Use your TraefikConfig type
  page: {
    title: string;
    actions: any[];  // Use your PageAction type
  };
}

interface StoredState {
  version: number;
  state: AppState;
}


// Load state from localStorage
function loadState(): AppState | null {
  if (!browser) return null;

  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const stored: StoredState = JSON.parse(saved);

      // Handle version migrations if needed
      if (stored.version < CURRENT_VERSION) {
        // Perform migration logic here
        console.log('Migrating from version', stored.version);
      }

      return stored.state;
    }
  } catch (e) {
    console.error('Failed to load state from localStorage:', e);
  }
  return null;
}

function saveState(state: AppState) {
  if (!browser) return;

  try {
    const storedState: StoredState = {
      version: CURRENT_VERSION,
      state
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(storedState));
  } catch (e) {
    console.error('Failed to save state to localStorage:', e);
  }
}


// Initialize state from localStorage
export function initializeStores() {
  const savedState = loadState();
  if (savedState) {
    if (savedState.config) {
      configStore.set(savedState.config);
    }
    if (savedState.page) {
      pageStore.set(savedState.page);
    }
  }
}

// Set up store subscriptions
if (browser) {
  // Create a debounced save function
  let saveTimeout: NodeJS.Timeout;
  function debouncedSave() {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      const state: AppState = {
        config: null,
        page: { title: '', actions: [] }
      };

      // Get latest values from stores
      configStore.subscribe(value => state.config = value)();
      pageStore.subscribe(value => state.page = value)();

      saveState(state);
    }, 1000); // Debounce for 1 second
  }

  // Subscribe to store changes
  configStore.subscribe(() => debouncedSave());
  pageStore.subscribe(() => debouncedSave());
}

export function clearStoredState() {
  if (browser) {
    localStorage.removeItem(STORAGE_KEY);
  }
}