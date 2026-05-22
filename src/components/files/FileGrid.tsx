import { useEffect, useRef, useState } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { DirectoryEntry } from '../../api/tauri';

const CELL_WIDTH = 120;
const CELL_HEIGHT = 132;
const MIN_COLUMN_GAP = 8;
const CONTAINER_PADDING = 8;

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
  const scrollRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(400);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerWidth(entry.contentRect.width);
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const columns = Math.max(1, Math.floor((containerWidth - CONTAINER_PADDING * 2 + MIN_COLUMN_GAP) / (CELL_WIDTH + MIN_COLUMN_GAP)));
  const rowCount = Math.ceil(entries.length / columns);

  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => CELL_HEIGHT,
    overscan: 3,
  });

  return (
    <div
      ref={scrollRef}
      data-testid="file-grid"
      aria-label="Icon grid view"
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
          padding: `${CONTAINER_PADDING}px`,
          boxSizing: 'border-box',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const rowStart = virtualRow.index * columns;
          const rowEnd = Math.min(rowStart + columns, entries.length);
          const rowEntries: DirectoryEntry[] = [];
          for (let c = rowStart; c < rowEnd; c++) {
            rowEntries.push(entries[c]);
          }

          return (
            <div
              key={virtualRow.key}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${CELL_HEIGHT}px`,
                transform: `translateY(${virtualRow.start}px)`,
                display: 'flex',
                gap: `${MIN_COLUMN_GAP}px`,
                padding: `0 ${CONTAINER_PADDING}px`,
                boxSizing: 'border-box',
              }}
            >
              {rowEntries.map((entry) => {
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
                      width: `${CELL_WIDTH}px`,
                      height: `${CELL_HEIGHT}px`,
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: 6,
                      padding: '12px 4px',
                      borderRadius: 'var(--radius-md)',
                      cursor: 'default',
                      background: isSelected ? 'var(--surface-floating)' : 'transparent',
                      border: `1px solid ${isSelected ? 'var(--border-strong)' : 'transparent'}`,
                      outline: 'none',
                      textAlign: 'center',
                      overflow: 'hidden',
                      boxSizing: 'border-box',
                      flexShrink: 0,
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
        })}
      </div>
    </div>
  );
}

export default FileGrid;