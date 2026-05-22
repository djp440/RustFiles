import { invoke } from '@tauri-apps/api/core';

export interface DirectoryEntry {
  path: string;
  name: string;
  size: number;
  modified: number;
  isHidden: boolean;
  isFolder: boolean;
}

export interface DirectoryPage {
  path: string;
  entries: DirectoryEntry[];
  totalCount: number;
  sortKey: 'name' | 'modified' | 'size' | 'type';
  sortAscending: boolean;
  filterKind: 'all' | 'folders' | 'files' | 'images' | 'documents';
  showHidden: boolean;
  snapshotVersion: number;
}

export interface SidebarRoots {
  desktop: string;
  downloads: string;
  documents: string;
  pictures: string;
  videos: string;
  music: string;
  thisPc: string;
}

interface RawSidebarRoots {
  desktop: string;
  downloads: string;
  documents: string;
  pictures: string;
  videos: string;
  music: string;
  this_pc?: string;
  thisPc?: string;
}

export interface DriveInfo {
  name: string;
  path: string;
  availableSpace: number;
  totalSpace: number;
}

export interface DriveList {
  drives: DriveInfo[];
}

const FALLBACK_ROOTS: SidebarRoots = {
  desktop: 'Desktop',
  downloads: 'Downloads',
  documents: 'Documents',
  pictures: 'Pictures',
  videos: 'Videos',
  music: 'Music',
  thisPc: 'This PC',
};

const FALLBACK_DRIVES: DriveList = {
  drives: [],
};

function toCamelDirectoryPage(page: {
  path: string;
  entries: DirectoryEntry[];
  total_count?: number;
  totalCount?: number;
  sort_key?: DirectoryPage['sortKey'];
  sortKey?: DirectoryPage['sortKey'];
  sort_ascending?: boolean;
  sortAscending?: boolean;
  filter_kind?: DirectoryPage['filterKind'];
  filterKind?: DirectoryPage['filterKind'];
  show_hidden?: boolean;
  showHidden?: boolean;
  snapshot_version?: number;
  snapshotVersion?: number;
}): DirectoryPage {
  return {
    path: page.path,
    entries: page.entries,
    totalCount: page.totalCount ?? page.total_count ?? page.entries.length,
    sortKey: page.sortKey ?? page.sort_key ?? 'name',
    sortAscending: page.sortAscending ?? page.sort_ascending ?? true,
    filterKind: page.filterKind ?? page.filter_kind ?? 'all',
    showHidden: page.showHidden ?? page.show_hidden ?? false,
    snapshotVersion: page.snapshotVersion ?? page.snapshot_version ?? 0,
  };
}

function toCamelSidebarRoots(roots: RawSidebarRoots): SidebarRoots {
  return {
    desktop: roots.desktop,
    downloads: roots.downloads,
    documents: roots.documents,
    pictures: roots.pictures,
    videos: roots.videos,
    music: roots.music,
    thisPc: roots.thisPc ?? roots.this_pc ?? 'This PC',
  };
}

export function hasTauriRuntime(): boolean {
  return typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
}

export async function listDirectory(path: string): Promise<DirectoryPage> {
  if (!hasTauriRuntime()) {
    return toCamelDirectoryPage({
      path,
      entries: [],
    });
  }

  try {
    const page = await invoke<{
      path: string;
      entries: DirectoryEntry[];
      total_count: number;
      sort_key: DirectoryPage['sortKey'];
      sort_ascending: boolean;
      filter_kind: DirectoryPage['filterKind'];
      show_hidden: boolean;
      snapshot_version: number;
    }>('list_directory', {
      path,
    });

    return toCamelDirectoryPage(page);
  } catch {
    return toCamelDirectoryPage({
      path,
      entries: [],
    });
  }
}

export async function getSidebarRoots(): Promise<SidebarRoots> {
  if (!hasTauriRuntime()) {
    return FALLBACK_ROOTS;
  }

  try {
    const roots = await invoke<RawSidebarRoots>('get_sidebar_roots');

    return toCamelSidebarRoots(roots);
  } catch {
    return FALLBACK_ROOTS;
  }
}

export async function getDrives(): Promise<DriveList> {
  if (!hasTauriRuntime()) {
    return FALLBACK_DRIVES;
  }

  try {
    return await invoke<DriveList>('get_drives');
  } catch {
    return FALLBACK_DRIVES;
  }
}
