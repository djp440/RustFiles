import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Settings } from '../api/tauri';
import { createSettingsStore } from '../stores/settings';

const { getSettingsMock, updateSettingsMock } = vi.hoisted(() => ({
  getSettingsMock: vi.fn(),
  updateSettingsMock: vi.fn(),
}));

vi.mock('../api/tauri', () => ({
  getSettings: getSettingsMock,
  updateSettings: updateSettingsMock,
}));

const baseSettings: Settings = {
  schemaVersion: 1,
  showHiddenFiles: false,
  showFileExtensions: true,
  sortKey: 'name',
  sortAscending: true,
};

function createStoreWithLoadedSettings(loadedSettings: Settings = baseSettings) {
  getSettingsMock.mockResolvedValue(loadedSettings);
  return createSettingsStore();
}

beforeEach(() => {
  getSettingsMock.mockReset();
  updateSettingsMock.mockReset();
});

describe('settings store', () => {
  it('writes loaded settings into the confirmed store state', async () => {
    const loadedSettings: Settings = {
      ...baseSettings,
      showHiddenFiles: true,
      sortKey: 'modified',
    };
    const store = createStoreWithLoadedSettings(loadedSettings);

    await store.loadSettings();

    expect(store.getState().confirmedSettings).toEqual(loadedSettings);
    expect(store.getState().settings).toEqual(loadedSettings);
    expect(store.getState().pending).toBe(false);
    expect(store.getState().error).toBeNull();
  });

  it('keeps save pending until Rust confirms the update', async () => {
    const confirmedSettings: Settings = {
      ...baseSettings,
      showHiddenFiles: false,
      showFileExtensions: true,
      sortKey: 'name',
      sortAscending: true,
    };
    const nextSettings: Settings = {
      schemaVersion: 1,
      showHiddenFiles: true,
      showFileExtensions: false,
      sortKey: 'size',
      sortAscending: false,
    };
    const savedSettings: Settings = {
      ...nextSettings,
    };
    const store = createStoreWithLoadedSettings(confirmedSettings);

    updateSettingsMock.mockResolvedValue(savedSettings);

    await store.loadSettings();

    const savePromise = store.saveSettings(nextSettings);

    expect(store.getState().pending).toBe(true);
    expect(store.getState().confirmedSettings).toEqual(confirmedSettings);
    expect(store.getState().settings).toEqual(nextSettings);

    await savePromise;

    expect(updateSettingsMock).toHaveBeenCalledWith(nextSettings);
    expect(store.getState().pending).toBe(false);
    expect(store.getState().confirmedSettings).toEqual(savedSettings);
    expect(store.getState().settings).toEqual(savedSettings);
    expect(store.getState().error).toBeNull();
  });

  it('rolls back to the last confirmed settings when Rust rejects the update', async () => {
    const confirmedSettings: Settings = {
      ...baseSettings,
      showHiddenFiles: false,
      showFileExtensions: true,
      sortKey: 'modified',
      sortAscending: true,
    };
    const nextSettings: Settings = {
      schemaVersion: 1,
      showHiddenFiles: true,
      showFileExtensions: false,
      sortKey: 'type',
      sortAscending: false,
    };
    const store = createStoreWithLoadedSettings(confirmedSettings);

    updateSettingsMock.mockRejectedValue(new Error('save failed'));

    await store.loadSettings();

    const savePromise = store.saveSettings(nextSettings);

    expect(store.getState().pending).toBe(true);
    expect(store.getState().settings).toEqual(nextSettings);
    expect(store.getState().settings.showHiddenFiles).toBe(true);
    expect(store.getState().settings.showFileExtensions).toBe(false);
    expect(store.getState().settings.sortKey).toBe('type');
    expect(store.getState().settings.sortAscending).toBe(false);

    await savePromise;

    expect(updateSettingsMock).toHaveBeenCalledWith(nextSettings);
    expect(store.getState().pending).toBe(false);
    expect(store.getState().confirmedSettings).toEqual(confirmedSettings);
    expect(store.getState().settings).toEqual(confirmedSettings);
    expect(store.getState().error).toBe('save failed');
    expect(store.getState().settings.showHiddenFiles).toBe(false);
    expect(store.getState().settings.showFileExtensions).toBe(true);
    expect(store.getState().settings.sortKey).toBe('modified');
    expect(store.getState().settings.sortAscending).toBe(true);
  });
});
