import type { DirectoryEntry } from '../../api/tauri';

interface FileListProps {
  entries: DirectoryEntry[];
  selectedPaths: Set<string>;
  showFileExtensions: boolean;
  onToggleSelect: (path: string) => void;
  onOpenEntry: (entry: DirectoryEntry) => void;
}

function FileList({
  entries,
  selectedPaths,
  showFileExtensions,
  onToggleSelect,
  onOpenEntry,
}: FileListProps) {
  return (
    <ul
      data-testid="file-list"
      aria-label="List view"
      role="list"
      style={{
        listStyle: 'none',
        padding: 0,
        margin: 0,
        display: 'grid',
        gap: 2,
      }}
    >
      {entries.map((entry) => {
        const isSelected = selectedPaths.has(entry.path);
        const displayName = showFileExtensions || entry.isFolder
          ? entry.name
          : entry.name.replace(/\.[^/.]+$/, "");

        return (
          <li
            key={entry.path}
            role="listitem"
            aria-label={entry.name}
            aria-selected={isSelected}
            onClick={(e) => {
              e.stopPropagation();
              onToggleSelect(entry.path);
            }}
            onDoubleClick={(e) => {
              e.stopPropagation();
              onOpenEntry(entry);
            }}
            style={{
              padding: '6px 12px',
              cursor: 'default',
              borderRadius: 'var(--radius-xs)',
              background: isSelected ? 'var(--surface-floating)' : 'transparent',
              border: `1px solid ${isSelected ? 'var(--border-strong)' : 'transparent'}`,
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              transition: 'background 0.2s',
            }}
          >
            <span
              aria-hidden="true"
              style={{
                fontSize: 16,
                opacity: 0.7,
              }}
            >
              {entry.isFolder ? '📁' : '📄'}
            </span>
            <span
              style={{
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
                color: isSelected ? 'var(--text-primary)' : 'var(--text-secondary)',
              }}
            >
              {displayName}
            </span>
            <span
              style={{
                marginLeft: 'auto',
                opacity: 0.6,
                fontSize: 12,
                color: 'var(--text-tertiary)',
              }}
            >
              {entry.isFolder ? 'Folder' : 'File'}
            </span>
          </li>
        );
      })}
    </ul>
  );
}

export default FileList;
