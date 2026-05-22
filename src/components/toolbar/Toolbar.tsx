import type { DirectoryPage } from '../../api/tauri';
import { useSearchStore } from '../../stores/search';

interface ToolbarProps {
  sortKey: DirectoryPage['sortKey'];
  sortAscending: boolean;
  filterKind: DirectoryPage['filterKind'];
  showHiddenFiles: boolean;
  showFileExtensions: boolean;
  onSortKeyChange: (sortKey: DirectoryPage['sortKey']) => void;
  onSortDirectionToggle: () => void;
  onFilterKindChange: (filterKind: DirectoryPage['filterKind']) => void;
  onShowHiddenFilesChange: (showHiddenFiles: boolean) => void;
  onShowFileExtensionsChange: (showFileExtensions: boolean) => void;
}

function Toolbar({
  sortKey,
  sortAscending,
  filterKind,
  showHiddenFiles,
  showFileExtensions,
  onSortKeyChange,
  onSortDirectionToggle,
  onFilterKindChange,
  onShowHiddenFilesChange,
  onShowFileExtensionsChange,
}: ToolbarProps) {
  const search = useSearchStore();
  const searchStatusLabel =
    search.status === 'idle'
      ? 'Idle'
      : search.status === 'queued'
        ? 'Queued'
        : search.status === 'running'
          ? 'Searching'
          : search.status === 'cancelling'
            ? 'Cancelling'
            : search.status === 'cancelled'
              ? 'Cancelled'
              : search.status === 'completed'
                ? 'Complete'
                : search.status === 'failed'
                  ? 'Failed'
                  : search.status === 'partially_completed'
                    ? 'Partial'
                    : 'Search';
  const showCancelButton = search.recursive && (search.status === 'queued' || search.status === 'running' || search.status === 'cancelling');

  return (
    <section
      aria-label="Toolbar"
      style={{
        display: 'flex',
        flexWrap: 'wrap',
        gap: 12,
        padding: '12px 16px',
        borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        alignItems: 'center',
      }}
    >
      <div style={{ display: 'grid', gap: 4, minWidth: 240, flex: '1 1 280px' }}>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <label style={{ display: 'grid', gap: 4, flex: '1 1 auto' }}>
            <span style={{ fontSize: 12 }}>Search</span>
            <input
              aria-label="Search files"
              value={search.query}
              onChange={(event) => search.setQuery(event.target.value)}
              placeholder="Search current folder"
            />
          </label>
          <button
            type="button"
            aria-label="Clear search"
            disabled={search.query.trim() === ''}
            onClick={() => search.clearSearch()}
          >
            Clear
          </button>
          {showCancelButton ? (
            <button
              type="button"
              aria-label="Cancel search"
              onClick={() => void search.cancelSearch()}
            >
              Cancel
            </button>
          ) : null}
        </div>
        <label style={{ display: 'inline-flex', gap: 8, alignItems: 'center' }}>
          <input
            aria-label="Recursive search"
            type="checkbox"
            checked={search.recursive}
            onChange={(event) => search.setRecursive(event.target.checked)}
          />
          <span>Recursive search</span>
          <span style={{ fontSize: 12, opacity: 0.7 }}>· {searchStatusLabel}</span>
        </label>
      </div>
      <label style={{ display: 'grid', gap: 4 }}>
        <span style={{ fontSize: 12 }}>Sort key</span>
        <select aria-label="Sort key" value={sortKey} onChange={(event) => onSortKeyChange(event.target.value as DirectoryPage['sortKey'])}>
          <option value="name">Name</option>
          <option value="modified">Modified</option>
          <option value="size">Size</option>
          <option value="type">Type</option>
        </select>
      </label>
      <button type="button" aria-label="Sort direction" onClick={onSortDirectionToggle}>
        {sortAscending ? 'Ascending' : 'Descending'}
      </button>
      <label style={{ display: 'grid', gap: 4 }}>
        <span style={{ fontSize: 12 }}>Filter kind</span>
        <select aria-label="Filter kind" value={filterKind} onChange={(event) => onFilterKindChange(event.target.value as DirectoryPage['filterKind'])}>
          <option value="all">All</option>
          <option value="folders">Folders</option>
          <option value="files">Files</option>
          <option value="images">Images</option>
          <option value="documents">Documents</option>
          <option value="videos">Videos</option>
        </select>
      </label>
      <label style={{ display: 'inline-flex', gap: 8, alignItems: 'center' }}>
        <input
          aria-label="Show hidden files"
          type="checkbox"
          checked={showHiddenFiles}
          onChange={(event) => onShowHiddenFilesChange(event.target.checked)}
        />
        <span>Show hidden files</span>
      </label>
      <label style={{ display: 'inline-flex', gap: 8, alignItems: 'center' }}>
        <input
          aria-label="Show file extensions"
          type="checkbox"
          checked={showFileExtensions}
          onChange={(event) => onShowFileExtensionsChange(event.target.checked)}
        />
        <span>Show file extensions</span>
      </label>
    </section>
  );
}

export default Toolbar;
