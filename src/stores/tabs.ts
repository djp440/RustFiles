import type { DirectoryEntry, DirectoryPage } from '../api/tauri';

export type { DirectoryEntry } from '../api/tauri';

export interface BreadcrumbSegment {
  label: string;
  path: string;
}

export interface TabState {
  id: string;
  path: string;
  history: string[];
  historyIndex: number;
  entries: DirectoryEntry[];
  loading: boolean;
  error: string | null;
}

function normalizePath(path: string): string {
  const trimmed = path.trim();
  if (trimmed === '') {
    return 'This PC';
  }

  if (/^[A-Za-z]:$/.test(trimmed)) {
    return `${trimmed}\\`;
  }

  return trimmed.replace(/[\\/]+$/, (suffix, offset, value) => {
    if (offset <= 2 && /^[A-Za-z]:\\$/.test(value)) {
      return suffix;
    }

    return '';
  });
}

function pushHistory(tab: TabState, nextPath: string): TabState {
  const normalizedPath = normalizePath(nextPath);
  if (normalizedPath === tab.path) {
    return {
      ...tab,
      loading: false,
      error: null,
    };
  }

  const history = tab.history.slice(0, tab.historyIndex + 1);
  history.push(normalizedPath);

  return {
    ...tab,
    path: normalizedPath,
    history,
    historyIndex: history.length - 1,
    loading: false,
    error: null,
  };
}

export function createTabState(initialPath: string): TabState {
  const path = normalizePath(initialPath);

  return {
    id: 'tab-1',
    path,
    history: [path],
    historyIndex: 0,
    entries: [],
    loading: false,
    error: null,
  };
}

export function setTabLoading(tab: TabState, loading: boolean): TabState {
  return {
    ...tab,
    loading,
    error: loading ? null : tab.error,
  };
}

export function setTabError(tab: TabState, error: string | null): TabState {
  return {
    ...tab,
    error,
    loading: false,
  };
}

export function applyDirectoryPage(tab: TabState, page: DirectoryPage): TabState {
  return {
    ...tab,
    entries: page.entries,
    loading: false,
    error: null,
  };
}

export function navigateTabToEntry(tab: TabState, entry: DirectoryEntry): TabState {
  if (!entry.isFolder) {
    return tab;
  }

  return pushHistory(tab, entry.path);
}

export function submitTabPath(tab: TabState, path: string): TabState {
  return pushHistory(tab, path);
}

export function navigateTabToBreadcrumb(tab: TabState, path: string): TabState {
  return pushHistory(tab, path);
}

export function goBackInTab(tab: TabState): TabState {
  if (tab.historyIndex === 0) {
    return tab;
  }

  const nextIndex = tab.historyIndex - 1;
  return {
    ...tab,
    path: tab.history[nextIndex],
    historyIndex: nextIndex,
    loading: false,
    error: null,
  };
}

export function goForwardInTab(tab: TabState): TabState {
  if (tab.historyIndex >= tab.history.length - 1) {
    return tab;
  }

  const nextIndex = tab.historyIndex + 1;
  return {
    ...tab,
    path: tab.history[nextIndex],
    historyIndex: nextIndex,
    loading: false,
    error: null,
  };
}

export function canGoBack(tab: TabState): boolean {
  return tab.historyIndex > 0;
}

export function canGoForward(tab: TabState): boolean {
  return tab.historyIndex < tab.history.length - 1;
}

export function getBreadcrumbSegments(path: string): BreadcrumbSegment[] {
  const normalizedPath = normalizePath(path);
  if (normalizedPath === 'This PC') {
    return [{ label: 'This PC', path: 'This PC' }];
  }

  const windowsMatch = normalizedPath.match(/^([A-Za-z]:\\)(.*)$/);
  if (!windowsMatch) {
    return [{ label: normalizedPath, path: normalizedPath }];
  }

  const [, driveRoot, remainder] = windowsMatch;
  const segments: BreadcrumbSegment[] = [{
    label: driveRoot.slice(0, 2),
    path: driveRoot.slice(0, 2),
  }];
  const parts = remainder.split('\\').filter(Boolean);

  let currentPath = driveRoot.replace(/\\$/, '');
  for (const part of parts) {
    currentPath = `${currentPath}\\${part}`;
    segments.push({
      label: part,
      path: currentPath,
    });
  }

  return segments;
}
