import { createElement } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import AppShell from '../components/shell/AppShell';
import FileBrowser from '../components/files/FileBrowser';
import { useInteractionReporting } from '../hooks/useInteractionReporting';
import {
  type DirectoryEntry,
  reportInteractionState,
  resetSchedulerDebugState,
  reportViewportState,
  type SchedulerDebugState,
  type SchedulerSignal,
} from '../api/tauri';

class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

const mocks = vi.hoisted(() => {
  type Settings = {
    schemaVersion: number;
    showHiddenFiles: boolean;
    showFileExtensions: boolean;
    sortKey: 'name' | 'modified' | 'size' | 'type';
    sortAscending: boolean;
  };

  const initialSettings = {
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

  const loadedSettings = {
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

  let settingsState = structuredClone(initialSettings);
  const subscribers = new Set<(state: typeof initialSettings) => void>();

  const notify = () => {
    for (const listener of subscribers) {
      listener(settingsState);
    }
  };

  const listDirectory = vi.fn(async (path: string, options?: Record<string, unknown>) => {
    if (!/^([A-Za-z]:\\|\\\\)/.test(path)) {
      return {
        path,
        entries: [],
        totalCount: 0,
        sortKey: (options?.sortKey as Settings['sortKey']) ?? 'name',
        sortAscending: (options?.sortAscending as boolean) ?? true,
        filterKind: (options?.filterKind as 'all' | 'folders' | 'files' | 'images' | 'documents' | 'videos') ?? 'all',
        showHidden: (options?.showHidden as boolean) ?? false,
        snapshotVersion: 1,
        offset: 0,
        limit: 0,
      };
    }

    const entries = Array.from({ length: 160 }, (_, index) => {
      const isFolder = index % 8 === 0;
      const name = isFolder ? `Projects-${index}` : `report-${index}.txt`;
      return {
        path: `${path}\\${name}`,
        name,
        size: isFolder ? 0 : 1024 + index,
        modified: 1_700_000_000 + index,
        isHidden: false,
        isFolder,
      };
    });

    return {
      path,
      entries,
      totalCount: entries.length,
      sortKey: (options?.sortKey as Settings['sortKey']) ?? 'name',
      sortAscending: (options?.sortAscending as boolean) ?? true,
      filterKind: (options?.filterKind as 'all' | 'folders' | 'files' | 'images' | 'documents' | 'videos') ?? 'all',
      showHidden: (options?.showHidden as boolean) ?? false,
      snapshotVersion: 1,
      offset: 0,
      limit: entries.length,
    };
  });

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
  const saveSettings = vi.fn(async (nextSettings: typeof initialSettings.settings) => {
    settingsState = {
      confirmedSettings: structuredClone(nextSettings),
      settings: structuredClone(nextSettings),
      pending: false,
      error: null,
    };
    notify();
  });
  const getSettingsState = vi.fn(() => settingsState);
  const subscribe = vi.fn((listener: (state: typeof initialSettings) => void) => {
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

vi.mock('../api/tauri', async () => {
  const actual = await vi.importActual<typeof import('../api/tauri')>('../api/tauri');

  return {
    ...actual,
    hasTauriRuntime: () => false,
    getSidebarRoots: mocks.getSidebarRoots,
    getDrives: mocks.getDrives,
    listDirectory: mocks.listDirectory,
  };
});

vi.mock('../stores/settings', () => ({
  loadSettings: mocks.loadSettings,
  saveSettings: mocks.saveSettings,
  getSettingsState: mocks.getSettingsState,
  settingsStore: {
    getState: mocks.getSettingsState,
    subscribe: mocks.subscribe,
  },
}));

function getSchedulerDebugState() {
  const debugState = (globalThis as typeof globalThis & {
    __RUSTFILES_SCHEDULER_DEBUG__?: SchedulerDebugState;
  }).__RUSTFILES_SCHEDULER_DEBUG__;

  expect(debugState).toBeDefined();
  return debugState as SchedulerDebugState;
}

function FileBrowserHarness({ entries }: { entries: DirectoryEntry[] }) {
  const { interactionEpoch, lastInputAtMs, reportInteraction } = useInteractionReporting({
    activeTabId: 'tab-1',
  });

  return (
    createElement(
      'div',
      { style: { height: '900px' } },
      createElement(FileBrowser, {
        path: 'C:\\Users\\demo',
        entries,
        loading: false,
        error: null,
        isTauriRuntime: false,
        activeTabId: 'tab-1',
        interactionEpoch,
        lastInputAtMs,
        onUserInteraction: () => void reportInteraction(),
        onOpenEntry: () => {},
      }),
    )
  );
}

beforeAll(() => {
  (globalThis as unknown as { ResizeObserver?: typeof ResizeObserver }).ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
    configurable: true,
    get() {
      return 600;
    },
  });
  Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
    configurable: true,
    get() {
      return 900;
    },
  });
  Object.defineProperty(HTMLElement.prototype, 'getBoundingClientRect', {
    configurable: true,
    value() {
      return {
        x: 0,
        y: 0,
        top: 0,
        left: 0,
        width: 900,
        height: 600,
        right: 900,
        bottom: 600,
        toJSON() {
          return this;
        },
      };
    },
  });
});

beforeEach(() => {
  mocks.reset();
  resetSchedulerDebugState();
  vi.restoreAllMocks();
});

describe('scheduler reporting', () => {
  it('records fallback ack data with stable observability keys', async () => {
    let now = 1_000;
    vi.spyOn(Date, 'now').mockImplementation(() => now);

    const viewportSignal: SchedulerSignal = {
      active_tab_id: 'tab-1',
      visible_range: {
        start_index: 0,
        end_index: 31,
      },
      last_input_at_ms: 984,
      is_scrolling: true,
      interaction_epoch: 7,
    };

    const viewportAck = await reportViewportState(viewportSignal);
    expect(viewportAck).toMatchObject({
      accepted: true,
      report_kind: 'viewport',
      active_tab_id: 'tab-1',
      interaction_epoch: 7,
      frame_budget_degraded: true,
      stale_interaction_epoch: false,
    });
    expect(viewportAck.interaction_latency_ms).toBe(16);
    expect(viewportAck.priority_order[0]).toBe('foreground_directory_enumeration');
    expect(viewportAck.priority_order[viewportAck.priority_order.length - 1]).toBe(
      'background_recursive_search',
    );

    now = 1_016;
    const interactionSignal: SchedulerSignal = {
      active_tab_id: 'tab-1',
      visible_range: null,
      last_input_at_ms: 1_016,
      is_scrolling: false,
      interaction_epoch: 8,
    };

    const interactionAck = await reportInteractionState(interactionSignal);
    expect(interactionAck).toMatchObject({
      accepted: true,
      report_kind: 'interaction',
      active_tab_id: 'tab-1',
      interaction_epoch: 8,
      frame_budget_degraded: false,
      stale_interaction_epoch: false,
    });
    expect(interactionAck.interaction_latency_ms).toBe(0);

    const debugState = getSchedulerDebugState();
    expect(debugState.reports).toHaveLength(2);
    expect(debugState.viewport_reports).toHaveLength(1);
    expect(debugState.interaction_reports).toHaveLength(1);
    expect(debugState.latest_viewport_ack?.interaction_latency_ms).toBe(16);
    expect(debugState.latest_interaction_ack?.interaction_latency_ms).toBe(0);
    expect(debugState.latest_interaction_ack?.frame_budget_degraded).toBe(false);
  });

  it('increments interaction epochs across navigation, selection, and view changes', async () => {
    render(
      createElement('div', { style: { height: '900px' } }, createElement(AppShell)),
    );

    await waitFor(() => expect(mocks.loadSettings).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByRole('button', { name: 'Desktop' }));

    await waitFor(() => expect(getSchedulerDebugState().interaction_reports.length).toBeGreaterThanOrEqual(1));

    const firstInteractionAck = getSchedulerDebugState().latest_interaction_ack;
    expect(firstInteractionAck?.interaction_epoch).toBe(1);

    fireEvent.change(screen.getByRole('textbox', { name: 'Path' }), {
      target: { value: 'C:\\Users\\demo' },
    });
    fireEvent.submit(screen.getByRole('textbox', { name: 'Path' }).closest('form') as HTMLFormElement);

    await waitFor(() => expect(mocks.listDirectory).toHaveBeenCalled());
    await waitFor(() => expect(getSchedulerDebugState().interaction_reports.length).toBeGreaterThanOrEqual(2));

    const secondInteractionAck = getSchedulerDebugState().latest_interaction_ack;
    expect(secondInteractionAck?.interaction_epoch).toBeGreaterThan(firstInteractionAck?.interaction_epoch ?? 0);

    await waitFor(() => expect(screen.getByRole('listitem', { name: 'Projects-0' })).toBeVisible());

    fireEvent.doubleClick(screen.getByRole('listitem', { name: 'Projects-0' }));

    await waitFor(() => expect(getSchedulerDebugState().interaction_reports.length).toBeGreaterThanOrEqual(3));

    const thirdInteractionAck = getSchedulerDebugState().latest_interaction_ack;
    expect(thirdInteractionAck?.interaction_epoch).toBeGreaterThan(secondInteractionAck?.interaction_epoch ?? 0);

    fireEvent.click(screen.getByRole('button', { name: 'Details view' }));

    await waitFor(() => expect(getSchedulerDebugState().interaction_reports.length).toBeGreaterThanOrEqual(4));

    const fourthInteractionAck = getSchedulerDebugState().latest_interaction_ack;
    expect(fourthInteractionAck?.interaction_epoch).toBeGreaterThan(thirdInteractionAck?.interaction_epoch ?? 0);
    expect(screen.getByTestId('details-table')).toBeInTheDocument();
  });

  it('reports visible ranges from the real virtual list and selection still works', async () => {
    render(
      createElement(FileBrowserHarness, {
        entries: Array.from({ length: 160 }, (_, index) => ({
          path: `C:\\Users\\demo\\${index % 8 === 0 ? `Projects-${index}` : `report-${index}.txt`}`,
          name: index % 8 === 0 ? `Projects-${index}` : `report-${index}.txt`,
          size: index % 8 === 0 ? 0 : 1024 + index,
          modified: 1_700_000_000 + index,
          isHidden: false,
          isFolder: index % 8 === 0,
        })),
      }),
    );

    await waitFor(() => expect(getSchedulerDebugState().viewport_reports.length).toBeGreaterThan(0));

    const initialViewportAck = getSchedulerDebugState().latest_viewport_ack;
    expect(initialViewportAck?.visible_range).toBeDefined();
    expect(initialViewportAck?.visible_range?.start_index).toBe(0);
    expect(initialViewportAck?.frame_budget_degraded).toBe(false);

    fireEvent.click(screen.getByRole('listitem', { name: 'Projects-0' }));

    await waitFor(() => expect(getSchedulerDebugState().interaction_reports.length).toBeGreaterThan(0));
    expect(getSchedulerDebugState().latest_interaction_ack?.interaction_epoch).toBe(1);

    fireEvent.click(screen.getByRole('button', { name: 'Details view' }));
    await waitFor(() => expect(screen.getByTestId('details-table')).toBeInTheDocument());

    fireEvent.click(screen.getByRole('button', { name: 'List view' }));
    await waitFor(() => expect(screen.getByTestId('file-list')).toBeInTheDocument());

    const scrollContainer = screen.getByTestId('file-list');
    scrollContainer.scrollTop = 1200;
    fireEvent.scroll(scrollContainer);

    await waitFor(() => expect(getSchedulerDebugState().viewport_reports.length).toBeGreaterThan(1));

    const scrolledViewportAck = getSchedulerDebugState().latest_viewport_ack;
    expect(scrolledViewportAck?.visible_range).toBeDefined();
    expect(scrolledViewportAck?.visible_range?.start_index).toBeGreaterThanOrEqual(0);
  });
});
