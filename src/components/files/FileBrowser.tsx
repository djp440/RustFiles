import type { DirectoryEntry } from '../../api/tauri';

interface FileBrowserProps {
  path: string;
  entries: DirectoryEntry[];
  loading: boolean;
  error: string | null;
  isTauriRuntime: boolean;
  sortKey?: 'name' | 'modified' | 'size' | 'type';
  sortAscending?: boolean;
  filterKind?: 'all' | 'folders' | 'files' | 'images' | 'documents' | 'videos';
  showHiddenFiles?: boolean;
  showFileExtensions?: boolean;
  onOpenEntry: (entry: DirectoryEntry) => void;
}

function isFilesystemPath(path: string): boolean {
  return /^[A-Za-z]:\\/.test(path) || path.startsWith('\\\\');
}

function getDisplayName(entry: DirectoryEntry, showFileExtensions: boolean): string {
  if (entry.isFolder || showFileExtensions) {
    return entry.name;
  }

  const lastDotIndex = entry.name.lastIndexOf('.');

  if (lastDotIndex <= 0) {
    return entry.name;
  }

  return entry.name.slice(0, lastDotIndex);
}

function formatSortKey(sortKey: FileBrowserProps['sortKey']): string {
  switch (sortKey) {
    case 'modified':
      return 'Modified';
    case 'size':
      return 'Size';
    case 'type':
      return 'Type';
    case 'name':
    default:
      return 'Name';
  }
}

function formatFilterKind(filterKind: FileBrowserProps['filterKind']): string {
  switch (filterKind) {
    case 'folders':
      return 'Folders';
    case 'files':
      return 'Files';
    case 'images':
      return 'Images';
    case 'documents':
      return 'Documents';
    case 'videos':
      return 'Videos';
    case 'all':
    default:
      return 'All';
  }
}

function FileBrowser({
  path,
  entries,
  loading,
  error,
  isTauriRuntime,
  sortKey = 'name',
  sortAscending = true,
  filterKind = 'all',
  showHiddenFiles = false,
  showFileExtensions = true,
  onOpenEntry,
}: FileBrowserProps) {
  const isPreviewPath = !isFilesystemPath(path);

  return (
    <section aria-label="File browser" style={{ padding: 16, display: 'grid', gap: 12 }}>
      <div>
        <strong>Current location:</strong> {path}
      </div>
      <div aria-label="View settings">
        View settings: {formatSortKey(sortKey)}, {sortAscending ? 'Ascending' : 'Descending'},{' '}
        {formatFilterKind(filterKind)}, Hidden {showHiddenFiles ? 'on' : 'off'}, Extensions{' '}
        {showFileExtensions ? 'on' : 'off'}
      </div>
      {loading ? <div>Loading directory...</div> : null}
      {error ? <div role="alert">{error}</div> : null}
      {!loading && !error && isPreviewPath ? (
        <div>
          {isTauriRuntime
            ? 'This location is a virtual entry. Select a drive or folder to browse files.'
            : 'This location is a preview entry. Browse real folders in the desktop app runtime.'}
        </div>
      ) : null}
      {!loading && !error && !isPreviewPath && entries.length === 0 ? <div>No items to show.</div> : null}
      <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'grid', gap: 8 }}>
        {entries.map((entry) => (
          <li key={entry.path}>
            <button
              type="button"
              onClick={() => onOpenEntry(entry)}
              disabled={!entry.isFolder}
              style={{ width: '100%', textAlign: 'left' }}
            >
              <span>{getDisplayName(entry, showFileExtensions)}</span>
              <span style={{ marginLeft: 8, opacity: 0.7 }}>
                {entry.isFolder ? 'Folder' : 'File'}
              </span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

export default FileBrowser;
