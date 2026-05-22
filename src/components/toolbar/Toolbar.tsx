import type { DirectoryPage } from '../../api/tauri';

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
