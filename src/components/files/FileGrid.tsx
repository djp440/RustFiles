import type { DirectoryEntry } from '../../api/tauri';

interface FileGridProps {
  entries: DirectoryEntry[];
  selectedPaths: Set<string>;
  showFileExtensions: boolean;
  onToggleSelect: (path: string) => void;
  onOpenEntry: (entry: DirectoryEntry) => void;
}

function FileGrid({
  entries,
  selectedPaths,
  showFileExtensions,
  onToggleSelect,
  onOpenEntry,
}: FileGridProps) {
  return (
    <div
      data-testid="file-grid"
      aria-label="Icon grid view"
      role="list"
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(100px, 1fr))',
        gap: 12,
        padding: 8,
      }}
    >
      {entries.map((entry) => {
        const isSelected = selectedPaths.has(entry.path);
        const displayName = showFileExtensions || entry.isFolder
          ? entry.name
          : entry.name.replace(/\.[^/.]+$/, "");

        return (
          <div
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
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: 6,
              padding: 12,
              borderRadius: 'var(--radius-md)',
              cursor: 'default',
              background: isSelected ? 'var(--surface-floating)' : 'transparent',
              border: `1px solid ${isSelected ? 'var(--border-strong)' : 'transparent'}`,
              outline: 'none',
              textAlign: 'center',
              overflow: 'hidden',
              transition: 'background 0.2s, border-color 0.2s',
            }}
          >
            <span
              aria-hidden="true"
              style={{
                fontSize: 32,
                lineHeight: 1,
                opacity: 0.8,
              }}
            >
              {entry.isFolder ? '📁' : '📄'}
            </span>
            <span
              style={{
                fontSize: 12,
                lineHeight: 1.3,
                wordBreak: 'break-all',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                display: '-webkit-box',
                WebkitLineClamp: 2,
                WebkitBoxOrient: 'vertical',
                maxWidth: '100%',
                color: isSelected ? 'var(--text-primary)' : 'var(--text-secondary)',
              }}
            >
              {displayName}
            </span>
          </div>
        );
      })}
    </div>
  );
}

export default FileGrid;
