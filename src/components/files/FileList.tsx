import { useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { DirectoryEntry } from '../../api/tauri';

const ROW_HEIGHT = 36;

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
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 5,
  });

  return (
    <div
      ref={scrollRef}
      data-testid="file-list"
      aria-label="List view"
      role="list"
      style={{
        overflow: 'auto',
        height: '100%',
      }}
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          position: 'relative',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => {
          const entry = entries[virtualItem.index];
          const isSelected = selectedPaths.has(entry.path);
          const displayName = showFileExtensions || entry.isFolder
            ? entry.name
            : entry.name.replace(/\.[^/.]+$/, "");

          return (
            <div
              key={virtualItem.key}
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
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${ROW_HEIGHT}px`,
                transform: `translateY(${virtualItem.start}px)`,
                padding: '6px 12px',
                cursor: 'default',
                borderRadius: 'var(--radius-xs)',
                background: isSelected ? 'var(--surface-floating)' : 'transparent',
                border: `1px solid ${isSelected ? 'var(--border-strong)' : 'transparent'}`,
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                boxSizing: 'border-box',
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
            </div>
          );
        })}
      </div>
    </div>
  );
}

export default FileList;