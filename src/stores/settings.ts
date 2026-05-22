import { getSettings, updateSettings, type Settings } from '../api/tauri';

export interface SettingsStoreState {
  confirmedSettings: Settings | null;
  settings: Settings;
  pending: boolean;
  error: string | null;
}

export interface SettingsStore {
  getState(): SettingsStoreState;
  subscribe(listener: (state: SettingsStoreState) => void): () => void;
  loadSettings(): Promise<void>;
  saveSettings(nextSettings: Settings): Promise<void>;
}

const DEFAULT_SETTINGS: Settings = {
  schemaVersion: 1,
  showHiddenFiles: false,
  showFileExtensions: true,
  sortKey: 'name',
  sortAscending: true,
};

function cloneSettings(settings: Settings): Settings {
  return { ...settings };
}

function createInitialState(): SettingsStoreState {
  return {
    confirmedSettings: null,
    settings: cloneSettings(DEFAULT_SETTINGS),
    pending: false,
    error: null,
  };
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim() !== '') {
    return error.message;
  }

  return 'Failed to save settings';
}

export function createSettingsStore(): SettingsStore {
  let state = createInitialState();
  const listeners = new Set<(state: SettingsStoreState) => void>();

  function notify() {
    for (const listener of listeners) {
      listener(state);
    }
  }

  function setState(nextState: SettingsStoreState) {
    state = nextState;
    notify();
  }

  return {
    getState() {
      return state;
    },

    subscribe(listener) {
      listeners.add(listener);

      return () => {
        listeners.delete(listener);
      };
    },

    async loadSettings() {
      try {
        const loadedSettings = await getSettings();

        setState({
          confirmedSettings: cloneSettings(loadedSettings),
          settings: cloneSettings(loadedSettings),
          pending: false,
          error: null,
        });
      } catch (error) {
        setState({
          ...state,
          pending: false,
          error: getErrorMessage(error),
        });
      }
    },

    async saveSettings(nextSettings) {
      const confirmedSettings = state.confirmedSettings ?? cloneSettings(state.settings);

      setState({
        confirmedSettings: state.confirmedSettings,
        settings: cloneSettings(nextSettings),
        pending: true,
        error: null,
      });

      try {
        const savedSettings = await updateSettings(nextSettings);

        setState({
          confirmedSettings: cloneSettings(savedSettings),
          settings: cloneSettings(savedSettings),
          pending: false,
          error: null,
        });
      } catch (error) {
        setState({
          confirmedSettings: cloneSettings(confirmedSettings),
          settings: cloneSettings(confirmedSettings),
          pending: false,
          error: getErrorMessage(error),
        });
      }
    },
  };
}

export const settingsStore = createSettingsStore();

export const loadSettings = settingsStore.loadSettings;
export const saveSettings = settingsStore.saveSettings;
export const getSettingsState = settingsStore.getState;
