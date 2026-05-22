import type { DirectoryEntry } from '../../api/tauri';

interface DetailsTableProps {
  entries: DirectoryEntry[];
  selectedPaths: Set<string>;
  showFileExtensions: boolean;
  onToggleSelect: (path: string) => void;
  onOpenEntry: (entry: DirectoryEntry) => void;
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '--';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

function formatDate(timestamp: number): string {
  if (timestamp === 0) return '--';
  return new Date(timestamp).toLocaleString();
}

function DetailsTable({
  entries,
  selectedPaths,
  showFileExtensions,
  onToggleSelect,
  onOpenEntry,
}: DetailsTableProps) {
  return (
    <table
      data-testid="details-table"
      style={{
        width: '100%',
        borderCollapse: 'collapse',
        fontSize: 13,
        textAlign: 'left',
      }}
    >
      <thead>
        <tr style={{ borderBottom: '1px solid var(--border-subtle)', color: 'var(--text-tertiary)' }}>
          <th style={{ padding: '8px 12px', fontWeight: 500 }}>Name</th>
          <th style={{ padding: '8px 12px', fontWeight: 500 }}>Date modified</th>
          <th style={{ padding: '8px 12px', fontWeight: 500 }}>Size</th>
        </tr>
      </thead>
      <tbody>
        {entries.map((entry) => {
          const isSelected = selectedPaths.has(entry.path);
          const displayName = showFileExtensions || entry.isFolder
            ? entry.name
            : entry.name.replace(/\.[^/.]+$/, "");

          return (
            <tr
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
                background: isSelected ? 'var(--surface-floating)' : 'transparent',
                borderBottom: '1px solid var(--border-subtle)',
                cursor: 'default',
                transition: 'background 0.2s',
              }}
            >
              <td style={{ padding: '8px 12px', display: 'flex', alignItems: 'center', gap: 8 }}>
                <span aria-hidden="true" style={{ opacity: 0.7 }}>
                  {entry.isFolder ? '📁' : '📄'}
                </span>
                <span style={{ color: isSelected ? 'var(--text-primary)' : 'var(--text-secondary)' }}>
                  {displayName}
                </span>
              </td>
              <td style={{ padding: '8px 12px', color: 'var(--text-tertiary)' }}>
                {formatDate(entry.modified)}
              </td>
              <td style={{ padding: '8px 12px', color: 'var(--text-tertiary)' }}>
                {entry.isFolder ? '--' : formatSize(entry.size)}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

export default DetailsTable;
