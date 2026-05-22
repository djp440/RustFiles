import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createSearchStore,
  flattenSearchResultBatches,
  getSearchResultParentPath,
  isSearchResultStale,
  type SearchResultBatch,
} from '../stores/search';

const mocks = vi.hoisted(() => {
  const startSearch = vi.fn(async () => 'search-task-1');
  const cancelTask = vi.fn(async () => 'cancelled');
  const listDirectory = vi.fn(async (path: string) => {
    const tree: Record<string, Array<{ path: string; name: string; isFolder: boolean }>> = {
      'C:\\Users\\demo': [
        { path: 'C:\\Users\\demo\\Projects', name: 'Projects', isFolder: true },
        { path: 'C:\\Users\\demo\\Documents', name: 'Documents', isFolder: true },
        { path: 'C:\\Users\\demo\\report-root.txt', name: 'report-root.txt', isFolder: false },
        { path: 'C:\\Users\\demo\\notes.txt', name: 'notes.txt', isFolder: false },
      ],
      'C:\\Users\\demo\\Projects': [
        { path: 'C:\\Users\\demo\\Projects\\RustFiles', name: 'RustFiles', isFolder: true },
        { path: 'C:\\Users\\demo\\Projects\\Archive', name: 'Archive', isFolder: true },
        { path: 'C:\\Users\\demo\\Projects\\report-project.txt', name: 'report-project.txt', isFolder: false },
      ],
      'C:\\Users\\demo\\Projects\\RustFiles': [
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src', name: 'src', isFolder: true },
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\docs', name: 'docs', isFolder: true },
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\search-notes.txt', name: 'search-notes.txt', isFolder: false },
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\report-plan.md', name: 'report-plan.md', isFolder: false },
      ],
      'C:\\Users\\demo\\Projects\\RustFiles\\src': [
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\search.ts', name: 'search.ts', isFolder: false },
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\search-store.ts', name: 'search-store.ts', isFolder: false },
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep', name: 'deep', isFolder: true },
      ],
      'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep': [
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep\\nested-report.txt', name: 'nested-report.txt', isFolder: false },
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep\\cancel-token.txt', name: 'cancel-token.txt', isFolder: false },
      ],
      'C:\\Users\\demo\\Projects\\RustFiles\\docs': [
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\docs\\search-spec.md', name: 'search-spec.md', isFolder: false },
        { path: 'C:\\Users\\demo\\Projects\\RustFiles\\docs\\recursion-notes.md', name: 'recursion-notes.md', isFolder: false },
      ],
      'C:\\Users\\demo\\Documents': [
        { path: 'C:\\Users\\demo\\Documents\\report-draft.txt', name: 'report-draft.txt', isFolder: false },
        { path: 'C:\\Users\\demo\\Documents\\todo.txt', name: 'todo.txt', isFolder: false },
      ],
      'C:\\Users\\demo\\Projects\\Archive': [
        { path: 'C:\\Users\\demo\\Projects\\Archive\\archived-report.txt', name: 'archived-report.txt', isFolder: false },
      ],
    };

    const entries = tree[path] ?? [];

    return {
      path,
      entries: entries.map((entry, index) => ({
        path: entry.path,
        name: entry.name,
        size: entry.isFolder ? 0 : 1024 + index,
        modified: 1_700_000_000 + index,
        isHidden: false,
        isFolder: entry.isFolder,
      })),
      totalCount: entries.length,
      sortKey: 'name',
      sortAscending: true,
      filterKind: 'all',
      showHidden: false,
      snapshotVersion: 100 + entries.length,
      offset: 0,
      limit: entries.length,
    };
  });

  return {
    startSearch,
    cancelTask,
    listDirectory,
  };
});

vi.mock('../api/tauri', async () => {
  const actual = await vi.importActual<typeof import('../api/tauri')>('../api/tauri');

  return {
    ...actual,
    hasTauriRuntime: () => true,
    startSearch: mocks.startSearch,
    cancelTask: mocks.cancelTask,
    listDirectory: mocks.listDirectory,
  };
});

const ROOT_ENTRIES = [
  {
    path: 'C:\\Users\\demo\\Projects',
    name: 'Projects',
    size: 0,
    modified: 1_700_000_000,
    isHidden: false,
    isFolder: true,
  },
  {
    path: 'C:\\Users\\demo\\Documents',
    name: 'Documents',
    size: 0,
    modified: 1_700_000_001,
    isHidden: false,
    isFolder: true,
  },
  {
    path: 'C:\\Users\\demo\\report-root.txt',
    name: 'report-root.txt',
    size: 1024,
    modified: 1_700_000_002,
    isHidden: false,
    isFolder: false,
  },
  {
    path: 'C:\\Users\\demo\\notes.txt',
    name: 'notes.txt',
    size: 2048,
    modified: 1_700_000_003,
    isHidden: false,
    isFolder: false,
  },
];

function createStore() {
  const store = createSearchStore();
  store.setContext({
    currentPath: 'C:\\Users\\demo',
    currentSnapshotVersion: 10,
    currentEntries: ROOT_ENTRIES,
    isTauriRuntime: true,
  });
  return store;
}

describe('search store', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
  });

  it('debounces search requests while updating visible results immediately', async () => {
    const store = createStore();

    store.setQuery('report');

    expect(store.getState().query).toBe('report');
    expect(store.getState().visibleEntries.map((entry) => entry.name)).toEqual(['report-root.txt']);
    expect(mocks.startSearch).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(179);
    expect(mocks.startSearch).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    await vi.runAllTicks();

    expect(mocks.startSearch).toHaveBeenCalledTimes(1);
    expect(mocks.startSearch).toHaveBeenCalledWith({
      root_path: 'C:\\Users\\demo',
      query: 'report',
      recursive: false,
      snapshot_version: 10,
    });
    expect(store.getState().status).toBe('completed');
    expect(store.getState().resultBatches).toHaveLength(1);
    expect(store.getState().resultBatches[0].matches.map((entry) => entry.name)).toEqual(['report-root.txt']);
  });

  it('keeps search results live when the current directory snapshot changes', async () => {
    const store = createStore();

    store.setQuery('report');
    await vi.advanceTimersByTimeAsync(200);
    await vi.runAllTicks();

    expect(store.getState().visibleEntries.map((entry) => entry.name)).toEqual(['report-root.txt']);

    store.setContext({
      currentPath: 'C:\\Users\\demo',
      currentSnapshotVersion: 11,
      currentEntries: [
        ...ROOT_ENTRIES,
        {
          path: 'C:\\Users\\demo\\report-extra.txt',
          name: 'report-extra.txt',
          size: 4096,
          modified: 1_700_000_004,
          isHidden: false,
          isFolder: false,
        },
      ],
      isTauriRuntime: true,
    });

    expect(store.getState().visibleEntries.map((entry) => entry.name)).toEqual([
      'report-root.txt',
      'report-extra.txt',
    ]);
  });

  it('clears the active query, results, and task state', async () => {
    const store = createStore();

    store.setQuery('report');
    await vi.advanceTimersByTimeAsync(200);
    await vi.runAllTicks();

    expect(store.getState().resultBatches).not.toHaveLength(0);

    store.clearSearch();

    expect(store.getState().query).toBe('');
    expect(store.getState().status).toBe('idle');
    expect(store.getState().taskId).toBeNull();
    expect(store.getState().error).toBeNull();
    expect(store.getState().resultBatches).toEqual([]);
    expect(store.getState().visibleEntries.map((entry) => entry.name)).toEqual(ROOT_ENTRIES.map((entry) => entry.name));
  });

  it('opens a live result at the correct parent path and refreshes stale results', async () => {
    const store = createStore();
    const batch: SearchResultBatch = {
      task_id: 'search-task-1',
      root_path: 'C:\\Users\\demo',
      query: 'search',
      recursive: true,
      snapshot_version: 10,
      matches: [
        {
          path: 'C:\\Users\\demo\\Projects\\RustFiles\\search-notes.txt',
          name: 'search-notes.txt',
          size: 1024,
          modified: 1_700_000_010,
          isHidden: false,
          isFolder: false,
        },
      ],
      error_summaries: [],
    };

    const [item] = flattenSearchResultBatches([batch]);
    const navigateToPath = vi.fn();
    const refreshCurrentDirectory = vi.fn();

    const liveOutcome = await store.openSearchResultLocation(item, {
      currentPath: 'C:\\Users\\demo',
      currentSnapshotVersion: 10,
      navigateToPath,
      refreshCurrentDirectory,
    });

    expect(liveOutcome.status).toBe('opened');
    expect(liveOutcome.targetPath).toBe('C:\\Users\\demo\\Projects\\RustFiles');
    expect(navigateToPath).toHaveBeenCalledWith('C:\\Users\\demo\\Projects\\RustFiles');
    expect(refreshCurrentDirectory).not.toHaveBeenCalled();
    expect(getSearchResultParentPath(item)).toBe('C:\\Users\\demo\\Projects\\RustFiles');

    const staleNavigateToPath = vi.fn();
    const staleRefreshCurrentDirectory = vi.fn();
    const staleOutcome = await store.openSearchResultLocation(item, {
      currentPath: 'C:\\Users\\demo\\Projects\\RustFiles',
      currentSnapshotVersion: 11,
      navigateToPath: staleNavigateToPath,
      refreshCurrentDirectory: staleRefreshCurrentDirectory,
    });

    expect(staleOutcome.status).toBe('missing');
    expect(staleOutcome.message).toBe('项目已不存在或已移动');
    expect(staleNavigateToPath).not.toHaveBeenCalled();
    expect(staleRefreshCurrentDirectory).toHaveBeenCalledTimes(1);
    expect(isSearchResultStale(item, 11, 'C:\\Users\\demo\\Projects\\RustFiles')).toBe(true);
  });
});
