import { useSyncExternalStore } from 'react';
import {
  cancelTask,
  hasTauriRuntime,
  listDirectory,
  startSearch,
  type DirectoryEntry,
  type SearchErrorSummary,
  type SearchRequest,
  type SearchResultBatch,
  type TaskStatus,
} from '../api/tauri';

export type { SearchResultBatch } from '../api/tauri';

const SEARCH_DEBOUNCE_MS = 180;
const SEARCH_BATCH_SIZE = 12;
const MISSING_LOCATION_ERROR = '项目已不存在或已移动';

export interface SearchResultItem {
  batch: SearchResultBatch;
  entry: DirectoryEntry;
  parentPath: string;
}

export interface SearchStoreContext {
  currentPath: string;
  currentSnapshotVersion: number | null;
  currentEntries: DirectoryEntry[];
  isTauriRuntime: boolean;
}

export interface SearchStoreState {
  query: string;
  recursive: boolean;
  status: TaskStatus | 'idle';
  taskId: string | null;
  error: string | null;
  currentPath: string;
  currentSnapshotVersion: number | null;
  currentEntries: DirectoryEntry[];
  visibleEntries: DirectoryEntry[];
  resultBatches: SearchResultBatch[];
}

export interface OpenSearchResultLocationOptions {
  currentPath: string;
  currentSnapshotVersion: number | null;
  navigateToPath: (path: string) => void | Promise<void>;
  refreshCurrentDirectory: () => void | Promise<void>;
}

export interface OpenSearchResultLocationOutcome {
  status: 'opened' | 'missing';
  targetPath: string;
  message?: string;
}

export interface SearchStore {
  getState(): SearchStoreState;
  subscribe(listener: (state: SearchStoreState) => void): () => void;
  setContext(context: SearchStoreContext): void;
  setQuery(query: string): void;
  setRecursive(recursive: boolean): void;
  clearSearch(): void;
  cancelSearch(): Promise<void>;
  openSearchResultLocation(
    item: SearchResultItem,
    options: OpenSearchResultLocationOptions,
  ): Promise<OpenSearchResultLocationOutcome>;
}

export interface SearchStoreHookSnapshot extends SearchStoreState {
  setContext: SearchStore['setContext'];
  setQuery: SearchStore['setQuery'];
  setRecursive: SearchStore['setRecursive'];
  clearSearch: SearchStore['clearSearch'];
  cancelSearch: SearchStore['cancelSearch'];
  openSearchResultLocation: SearchStore['openSearchResultLocation'];
}

interface SearchExecutionContext {
  token: number;
  query: string;
  recursive: boolean;
  currentPath: string;
  currentSnapshotVersion: number | null;
  currentEntries: DirectoryEntry[];
  isTauriRuntime: boolean;
}

function cloneEntries(entries: DirectoryEntry[]): DirectoryEntry[] {
  return entries.map((entry) => ({ ...entry }));
}

function createEmptyBatch(context: SearchExecutionContext, taskId: string): SearchResultBatch {
  return {
    task_id: taskId,
    root_path: context.currentPath,
    query: context.query,
    recursive: context.recursive,
    snapshot_version: context.currentSnapshotVersion,
    matches: [],
    error_summaries: [],
  };
}

function isFilesystemPath(path: string): boolean {
  return /^[A-Za-z]:\\/.test(path) || path.startsWith('\\\\');
}

function normalizeQuery(query: string): string {
  return query.trim().toLowerCase();
}

function entryMatches(entry: DirectoryEntry, query: string): boolean {
  if (!query) {
    return true;
  }

  return entry.name.toLowerCase().includes(query);
}

function getParentPath(path: string): string {
  const normalizedPath = path.trim();
  if (normalizedPath === '' || normalizedPath === 'This PC') {
    return normalizedPath || 'This PC';
  }

  const windowsMatch = normalizedPath.match(/^([A-Za-z]:\\)(.*)$/);
  if (windowsMatch) {
    const [, driveRoot, remainder] = windowsMatch;
    const parts = remainder.split('\\').filter(Boolean);
    if (parts.length === 0) {
      return driveRoot;
    }

    parts.pop();
    if (parts.length === 0) {
      return driveRoot;
    }

    return `${driveRoot}${parts.join('\\')}`;
  }

  const slashIndex = normalizedPath.lastIndexOf('\\');
  if (slashIndex <= 0) {
    return normalizedPath;
  }

  return normalizedPath.slice(0, slashIndex);
}

function buildCurrentDirectoryBatch(context: SearchExecutionContext, taskId: string): SearchResultBatch {
  const matches = context.currentEntries.filter((entry) => entryMatches(entry, normalizeQuery(context.query)));

  return {
    ...createEmptyBatch(context, taskId),
    matches,
  };
}

function createInitialState(): SearchStoreState {
  return {
    query: '',
    recursive: false,
    status: 'idle',
    taskId: null,
    error: null,
    currentPath: 'This PC',
    currentSnapshotVersion: null,
    currentEntries: [],
    visibleEntries: [],
    resultBatches: [],
  };
}

function makeAppError(message: string): SearchErrorSummary['error'] {
  return {
    code: 'internal_error',
    message,
    retryable: false,
    refresh_suggestion: null,
  };
}

export function flattenSearchResultBatches(batches: SearchResultBatch[]): SearchResultItem[] {
  return batches.flatMap((batch) =>
    batch.matches.map((entry) => ({
      batch,
      entry,
      parentPath: getParentPath(entry.path),
    })),
  );
}

export function getSearchResultParentPath(item: SearchResultItem): string {
  return item.parentPath;
}

export function isSearchResultStale(
  item: SearchResultItem,
  currentSnapshotVersion: number | null,
  currentPath?: string,
): boolean {
  if (currentPath && currentPath !== item.parentPath) {
    return false;
  }

  if (currentPath && currentPath === item.parentPath && item.batch.recursive) {
    return true;
  }

  if (currentSnapshotVersion === null) {
    return true;
  }

  return item.batch.snapshot_version !== currentSnapshotVersion;
}

export function createSearchStore(): SearchStore {
  let state = createInitialState();
  const listeners = new Set<(state: SearchStoreState) => void>();
  let executionContext: SearchExecutionContext = {
    token: 0,
    query: '',
    recursive: false,
    currentPath: 'This PC',
    currentSnapshotVersion: null,
    currentEntries: [],
    isTauriRuntime: false,
  };
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let activeCancellationRequested = false;
  let lastOpenedResultPath: string | null = null;

  function notify() {
    for (const listener of listeners) {
      listener(state);
    }
  }

  function commit(nextState: Partial<SearchStoreState>) {
    state = {
      ...state,
      ...nextState,
    };
    notify();
  }

  function clearTimer() {
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
  }

  function invalidateExecution() {
    executionContext = {
      ...executionContext,
      token: executionContext.token + 1,
    };
    activeCancellationRequested = true;
    clearTimer();
  }

  function updateImmediateResults(taskId: string) {
    const batch = buildCurrentDirectoryBatch(executionContext, taskId);
    commit({
      visibleEntries: cloneEntries(batch.matches),
      resultBatches: batch.matches.length > 0 ? [batch] : [],
      error: state.error === MISSING_LOCATION_ERROR ? state.error : null,
    });
  }

  async function startBackendSearch(): Promise<string | null> {
    if (!executionContext.isTauriRuntime || !hasTauriRuntime()) {
      return null;
    }

    const request: SearchRequest = {
      root_path: executionContext.currentPath,
      query: executionContext.query,
      recursive: executionContext.recursive,
      snapshot_version: executionContext.currentSnapshotVersion,
    };

    try {
      const taskId = await startSearch(request);
      commit({
        taskId,
      });
      return taskId;
    } catch (error) {
      const message = error instanceof Error && error.message.trim() !== '' ? error.message : '搜索任务启动失败';
      commit({
        status: 'failed',
        error: message,
      });
      return null;
    }
  }

  async function performRecursiveSearch(taskId: string, searchToken: number): Promise<void> {
    const query = normalizeQuery(executionContext.query);
    const batchByDirectory = new Map<string, SearchResultBatch>();
    const rootBatch = buildCurrentDirectoryBatch(executionContext, taskId);
    const resultBatches: SearchResultBatch[] = rootBatch.matches.length > 0 ? [rootBatch] : [];
    let hadMatchesOrErrors = rootBatch.matches.length > 0;
    let processedCount = 0;
    const stack: string[] = isFilesystemPath(executionContext.currentPath)
      ? [executionContext.currentPath]
      : [];

    commit({
      status: 'running',
      taskId,
      resultBatches,
      visibleEntries: cloneEntries(rootBatch.matches),
      error: state.error === MISSING_LOCATION_ERROR ? state.error : null,
    });

    while (stack.length > 0) {
      if (executionContext.token !== searchToken || activeCancellationRequested) {
        break;
      }

      const directoryPath = stack.pop()!;

      let page;
      try {
        page = await listDirectory(directoryPath, {
          sortKey: 'name',
          sortAscending: true,
          filterKind: 'all',
          showHidden: true,
        });
      } catch (error) {
        hadMatchesOrErrors = true;
        const batch = batchByDirectory.get(directoryPath) ?? {
          ...createEmptyBatch(executionContext, taskId),
          root_path: directoryPath,
        };
        batch.error_summaries.push({
          path: directoryPath,
          error: makeAppError(error instanceof Error ? error.message : '读取目录失败'),
        });
        batchByDirectory.set(directoryPath, batch);
        if (!resultBatches.includes(batch)) {
          resultBatches.push(batch);
        }
        continue;
      }

      const batch = batchByDirectory.get(directoryPath) ?? {
        task_id: taskId,
        root_path: directoryPath,
        query: executionContext.query,
        recursive: true,
        snapshot_version: executionContext.currentSnapshotVersion,
        matches: [],
        error_summaries: [],
      };

      for (const entry of page.entries) {
        if (executionContext.token !== searchToken || activeCancellationRequested) {
          break;
        }

        if (entryMatches(entry, query)) {
          hadMatchesOrErrors = true;
          batch.matches.push(entry);
          if (batch.matches.length >= SEARCH_BATCH_SIZE) {
            resultBatches.push({
              ...batch,
              matches: cloneEntries(batch.matches),
              error_summaries: batch.error_summaries.map((item) => ({ ...item })),
            });
            batch.matches = [];
            batch.error_summaries = [];
          }
        }

        if (entry.isFolder) {
          stack.push(entry.path);
        }

        processedCount += 1;
        if (processedCount % 12 === 0) {
          await new Promise((resolve) => setTimeout(resolve, 5));
        }
      }

      if (batch.matches.length > 0 || batch.error_summaries.length > 0) {
        resultBatches.push({
          ...batch,
          matches: cloneEntries(batch.matches),
          error_summaries: batch.error_summaries.map((item) => ({ ...item })),
        });
      }

      commit({
        resultBatches: resultBatches.map((item) => ({
          ...item,
          matches: cloneEntries(item.matches),
          error_summaries: item.error_summaries.map((summary) => ({ ...summary, error: { ...summary.error } })),
        })),
      });
    }

    if (executionContext.token !== searchToken || activeCancellationRequested) {
      commit({
        status: hadMatchesOrErrors ? 'partially_completed' : 'cancelled',
      });
      return;
    }

    commit({
      status: 'completed',
      resultBatches: resultBatches.map((item) => ({
        ...item,
        matches: cloneEntries(item.matches),
        error_summaries: item.error_summaries.map((summary) => ({ ...summary, error: { ...summary.error } })),
      })),
    });
  }

  async function runSearch(): Promise<void> {
    const searchToken = executionContext.token;
    const taskId = (await startBackendSearch()) ?? `preview-search-${Date.now()}`;
    if (executionContext.token !== searchToken || activeCancellationRequested) {
      return;
    }

    commit({
      taskId,
    });

    updateImmediateResults(taskId);

    if (!executionContext.recursive) {
      commit({
        status: 'completed',
        taskId,
      });
      return;
    }

    await performRecursiveSearch(taskId, searchToken);
  }

  function scheduleSearch() {
    clearTimer();
    activeCancellationRequested = false;

    if (state.query.trim() === '') {
      commit({
        status: 'idle',
        taskId: null,
        error: null,
        resultBatches: [],
        visibleEntries: cloneEntries(executionContext.currentEntries),
      });
      invalidateExecution();
      return;
    }

    const normalizedQuery = normalizeQuery(state.query);
    const previewTaskId = `preview-search-${Date.now()}`;
    const batch = buildCurrentDirectoryBatch(
      {
        ...executionContext,
        query: normalizedQuery,
      },
      previewTaskId,
    );

    commit({
      status: 'queued',
      error: state.error === MISSING_LOCATION_ERROR ? state.error : null,
      visibleEntries: cloneEntries(batch.matches),
      resultBatches: batch.matches.length > 0 ? [batch] : [],
    });

    clearTimer();
    debounceTimer = setTimeout(() => {
      void runSearch();
    }, SEARCH_DEBOUNCE_MS);
  }

  function syncContext(nextContext: SearchStoreContext) {
    executionContext = {
      ...executionContext,
      currentPath: nextContext.currentPath,
      currentSnapshotVersion: nextContext.currentSnapshotVersion,
      currentEntries: cloneEntries(nextContext.currentEntries),
      isTauriRuntime: nextContext.isTauriRuntime,
    };

    commit({
      currentPath: nextContext.currentPath,
      currentSnapshotVersion: nextContext.currentSnapshotVersion,
      currentEntries: cloneEntries(nextContext.currentEntries),
      visibleEntries: state.query.trim() === '' ? cloneEntries(nextContext.currentEntries) : cloneEntries(nextContext.currentEntries.filter((entry) => entryMatches(entry, normalizeQuery(state.query)))),
    });

    if (state.query.trim() !== '') {
      invalidateExecution();
      scheduleSearch();
    }
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

    setContext(context) {
      syncContext(context);
    },

    setQuery(query) {
      lastOpenedResultPath = null;
      state = {
        ...state,
        query,
      };
      executionContext = {
        ...executionContext,
        query,
      };
      notify();
      scheduleSearch();
    },

    setRecursive(recursive) {
      if (state.recursive === recursive) {
        return;
      }

      lastOpenedResultPath = null;
      state = {
        ...state,
        recursive,
      };
      notify();

      executionContext = {
        ...executionContext,
        recursive,
      };

      if (state.query.trim() !== '') {
        invalidateExecution();
        scheduleSearch();
      }
    },

    clearSearch() {
      invalidateExecution();
      executionContext = {
        ...executionContext,
        query: '',
      };
      lastOpenedResultPath = null;
      commit({
        query: '',
        status: 'idle',
        taskId: null,
        error: null,
        resultBatches: [],
        visibleEntries: cloneEntries(executionContext.currentEntries),
      });
    },

    async cancelSearch() {
      const currentTaskId = state.taskId;
      invalidateExecution();
      activeCancellationRequested = true;

      if (currentTaskId !== null) {
        commit({
          status: 'cancelling',
        });
        try {
          await cancelTask(currentTaskId);
        } catch {
          // Keep the front-end state cancellable even when the backend cancel fails.
        }
      }

      commit({
        status: state.resultBatches.length > 0 ? 'partially_completed' : 'cancelled',
        taskId: null,
      });
      activeCancellationRequested = false;
    },

    async openSearchResultLocation(item, options) {
      const repeatedOpen = lastOpenedResultPath === item.entry.path;
      const stale = isSearchResultStale(item, options.currentSnapshotVersion, options.currentPath);
      const targetPath = getParentPath(item.entry.path);

      if (stale || repeatedOpen) {
        const message = '项目已不存在或已移动';
        commit({
          error: message,
        });
        await options.refreshCurrentDirectory();
        return {
          status: 'missing',
          targetPath,
          message,
        };
      }

      commit({
        error: null,
      });
      lastOpenedResultPath = item.entry.path;
      await options.navigateToPath(targetPath);
      return {
        status: 'opened',
        targetPath,
      };
    },
  };
}

export const searchStore = createSearchStore();

export function useSearchStore(): SearchStoreHookSnapshot {
  const state = useSyncExternalStore(
    searchStore.subscribe,
    searchStore.getState,
    searchStore.getState,
  );

  return {
    ...state,
    setContext: searchStore.setContext,
    setQuery: searchStore.setQuery,
    setRecursive: searchStore.setRecursive,
    clearSearch: searchStore.clearSearch,
    cancelSearch: searchStore.cancelSearch,
    openSearchResultLocation: searchStore.openSearchResultLocation,
  };
}
