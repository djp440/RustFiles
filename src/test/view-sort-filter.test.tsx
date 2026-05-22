import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import AppShell from '../components/shell/AppShell';
import FileBrowser from '../components/files/FileBrowser';

type SettingsState = {
  confirmedSettings: {
    schemaVersion: number;
    showHiddenFiles: boolean;
    showFileExtensions: boolean;
    sortKey: 'name' | 'modified' | 'size' | 'type';
    sortAscending: boolean;
  };
  settings: {
    schemaVersion: number;
    showHiddenFiles: boolean;
    showFileExtensions: boolean;
    sortKey: 'name' | 'modified' | 'size' | 'type';
    sortAscending: boolean;
  };
  pending: boolean;
  error: string | null;
};

const mocks = vi.hoisted(() => {
  const initialSettings: SettingsState = {
    confirmedSettings: {
      schemaVersion: 1,
      showHiddenFiles: false,
      showFileExtensions: true,
      sortKey: 'name',
      sortAscending: true,
    },
    settings: {
      schemaVersion: 1,
      showHiddenFiles: false,
      showFileExtensions: true,
      sortKey: 'name',
      sortAscending: true,
    },
    pending: false,
    error: null,
  };

  const loadedSettings: SettingsState = {
    confirmedSettings: {
      schemaVersion: 1,
      showHiddenFiles: true,
      showFileExtensions: false,
      sortKey: 'modified',
      sortAscending: false,
    },
    settings: {
      schemaVersion: 1,
      showHiddenFiles: true,
      showFileExtensions: false,
      sortKey: 'modified',
      sortAscending: false,
    },
    pending: false,
    error: null,
  };

  let settingsState: SettingsState = structuredClone(initialSettings);
  const subscribers = new Set<(state: SettingsState) => void>();

  const notify = () => {
    for (const listener of subscribers) {
      listener(settingsState);
    }
  };

  const cloneSettings = (settings: SettingsState['settings']) => structuredClone(settings);

  const listDirectory = vi.fn(async (path: string, options?: Record<string, unknown>) => ({
    path,
    entries: [
      {
        path: `${path}\\Projects`,
        name: 'Projects',
        size: 0,
        modified: 0,
        isHidden: false,
        isFolder: true,
      },
      {
        path: `${path}\\report.txt`,
        name: 'report.txt',
        size: 128,
        modified: 1,
        isHidden: false,
        isFolder: false,
      },
    ],
    totalCount: 2,
    sortKey: (options?.sortKey as 'name' | 'modified' | 'size' | 'type') ?? 'name',
    sortAscending: (options?.sortAscending as boolean) ?? true,
    filterKind: (options?.filterKind as 'all' | 'folders' | 'files' | 'images' | 'documents' | 'videos') ?? 'all',
    showHidden: (options?.showHidden as boolean) ?? false,
    snapshotVersion: 1,
  }));

  const getSidebarRoots = vi.fn(async () => ({
    desktop: 'C:\\Users\\RustFiles\\Desktop',
    downloads: 'C:\\Users\\RustFiles\\Downloads',
    documents: 'C:\\Users\\RustFiles\\Documents',
    pictures: 'C:\\Users\\RustFiles\\Pictures',
    videos: 'C:\\Users\\RustFiles\\Videos',
    music: 'C:\\Users\\RustFiles\\Music',
    thisPc: 'This PC',
  }));

  const getDrives = vi.fn(async () => ({ drives: [] }));
  const loadSettings = vi.fn(async () => {
    settingsState = structuredClone(loadedSettings);
    notify();
  });
  const saveSettings = vi.fn(async (nextSettings: SettingsState['settings']) => {
    settingsState = {
      confirmedSettings: cloneSettings(nextSettings),
      settings: cloneSettings(nextSettings),
      pending: false,
      error: null,
    };
    notify();
  });
  const getSettingsState = vi.fn(() => settingsState);
  const subscribe = vi.fn((listener: (state: SettingsState) => void) => {
    subscribers.add(listener);
    return () => subscribers.delete(listener);
  });

  const reset = () => {
    settingsState = structuredClone(initialSettings);
    subscribers.clear();
    listDirectory.mockClear();
    getSidebarRoots.mockClear();
    getDrives.mockClear();
    loadSettings.mockClear();
    saveSettings.mockClear();
    getSettingsState.mockClear();
    subscribe.mockClear();
  };

  return {
    listDirectory,
    getSidebarRoots,
    getDrives,
    loadSettings,
    saveSettings,
    getSettingsState,
    subscribe,
    reset,
  };
});

vi.mock('../api/tauri', () => ({
  hasTauriRuntime: () => true,
  getSidebarRoots: mocks.getSidebarRoots,
  getDrives: mocks.getDrives,
  listDirectory: mocks.listDirectory,
}));

vi.mock('../stores/settings', () => ({
  loadSettings: mocks.loadSettings,
  saveSettings: mocks.saveSettings,
  getSettingsState: mocks.getSettingsState,
  settingsStore: {
    getState: mocks.getSettingsState,
    subscribe: mocks.subscribe,
  },
}));

function selectSidebarRoot(label: string) {
  fireEvent.click(screen.getByRole('button', { name: label }));
}

function getToolbar() {
  return screen.getByLabelText('Toolbar');
}

function getLastDirectoryRequest() {
  const call = mocks.listDirectory.mock.calls[mocks.listDirectory.mock.calls.length - 1];

  if (!call) {
    throw new Error('Expected listDirectory to be called');
  }

  return call;
}

beforeEach(() => {
  mocks.reset();
});

describe('view, sort, and filter wiring', () => {
  it('uses loaded settings for the first directory request', async () => {
    render(<AppShell />);

    await waitFor(() => expect(mocks.loadSettings).toHaveBeenCalledTimes(1));

    selectSidebarRoot('Desktop');

    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalledTimes(1));

    expect(getLastDirectoryRequest()).toEqual([
      'C:\\Users\\RustFiles\\Desktop',
      {
        sortKey: 'modified',
        sortAscending: false,
        filterKind: 'all',
        showHidden: true,
      },
    ]);
  });

  it('updates the next directory request when the sort key changes', async () => {
    render(<AppShell />);

    await waitFor(() => expect(mocks.loadSettings).toHaveBeenCalledTimes(1));
    selectSidebarRoot('Desktop');
    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalledTimes(1));

    fireEvent.change(within(getToolbar()).getByLabelText('Sort key'), {
      target: { value: 'size' },
    });

    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalledTimes(2));

    expect(getLastDirectoryRequest()[1]).toMatchObject({
      sortKey: 'size',
      sortAscending: false,
      filterKind: 'all',
      showHidden: true,
    });
  });

  it('updates the next directory request when the filter kind changes', async () => {
    render(<AppShell />);

    await waitFor(() => expect(mocks.loadSettings).toHaveBeenCalledTimes(1));
    selectSidebarRoot('Desktop');
    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalledTimes(1));

    fireEvent.change(within(getToolbar()).getByLabelText('Filter kind'), {
      target: { value: 'videos' },
    });

    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalledTimes(2));

    expect(getLastDirectoryRequest()[1]).toMatchObject({
      sortKey: 'modified',
      sortAscending: false,
      filterKind: 'videos',
      showHidden: true,
    });
  });

  it('updates the next directory request when hidden files are shown', async () => {
    render(<AppShell />);

    await waitFor(() => expect(mocks.loadSettings).toHaveBeenCalledTimes(1));
    selectSidebarRoot('Desktop');
    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalledTimes(1));

    fireEvent.click(within(getToolbar()).getByLabelText('Show hidden files'));

    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalledTimes(2));

    expect(getLastDirectoryRequest()[1]).toMatchObject({
      sortKey: 'modified',
      sortAscending: false,
      filterKind: 'all',
      showHidden: false,
    });
  });

  it('hides file extensions without changing folder names', () => {
    render(
      <FileBrowser
        path="C:\\Users\\RustFiles\\Desktop"
        entries={[
          {
            path: 'C:\\Users\\RustFiles\\Desktop\\Projects',
            name: 'Projects',
            size: 0,
            modified: 0,
            isHidden: false,
            isFolder: true,
          },
          {
            path: 'C:\\Users\\RustFiles\\Desktop\\report.txt',
            name: 'report.txt',
            size: 128,
            modified: 1,
            isHidden: false,
            isFolder: false,
          },
        ]}
        loading={false}
        error={null}
        isTauriRuntime
        showFileExtensions={false}
        onOpenEntry={() => {}}
      />,
    );

    expect(screen.getByText('Projects')).toBeInTheDocument();
    expect(screen.getByText('report')).toBeInTheDocument();
    expect(screen.queryByText('report.txt')).not.toBeInTheDocument();
  });
});
