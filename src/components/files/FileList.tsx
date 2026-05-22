import { useEffect, useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { DirectoryEntry } from '../../api/tauri';
import { useViewportReporting } from '../../hooks/useViewportReporting';

const ROW_HEIGHT = 36;

interface FileListProps {
  entries: DirectoryEntry[];
  selectedPaths: Set<string>;
  showFileExtensions: boolean;
  onToggleSelect: (path: string) => void;
  onOpenEntry: (entry: DirectoryEntry) => void;
  activeTabId?: string;
  interactionEpoch?: number;
  lastInputAtMs?: number | null;
}

function FileList({
  entries,
  selectedPaths,
  showFileExtensions,
  onToggleSelect,
  onOpenEntry,
  activeTabId = 'tab-1',
  interactionEpoch = 0,
  lastInputAtMs = null,
}: FileListProps) {
  const { scrollRef, handleScroll, setVisibleRange } = useViewportReporting({
    activeTabId,
    interactionEpoch,
    lastInputAtMs,
    visibleRange: null,
  });

  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 5,
  });
  const virtualItems = virtualizer.getVirtualItems();
  const fallbackStartIndex = Math.floor((scrollRef.current?.scrollTop ?? 0) / ROW_HEIGHT);
  const fallbackVisibleCount = Math.max(1, Math.ceil((scrollRef.current?.clientHeight ?? 0) / ROW_HEIGHT));
  const visibleRange =
    virtualItems.length > 0
      ? {
          start_index: virtualItems[0].index,
          end_index: virtualItems[virtualItems.length - 1].index,
        }
      : entries.length > 0
        ? {
            start_index: Math.min(Math.max(0, fallbackStartIndex), entries.length - 1),
            end_index: Math.min(entries.length - 1, Math.max(0, fallbackStartIndex) + fallbackVisibleCount - 1),
          }
        : null;
  const previousVisibleRangeRef = useRef<typeof visibleRange>(null);
  const renderedItems =
    virtualItems.length > 0
      ? virtualItems
      : visibleRange
        ? Array.from(
            { length: visibleRange.end_index - visibleRange.start_index + 1 },
            (_, offset) => {
              const index = visibleRange.start_index + offset;
              return {
                key: `fallback-${index}`,
                index,
                start: index * ROW_HEIGHT,
              };
            },
          )
        : [];

  useEffect(() => {
    const previousVisibleRange = previousVisibleRangeRef.current;
    if (
      previousVisibleRange?.start_index === visibleRange?.start_index &&
      previousVisibleRange?.end_index === visibleRange?.end_index
    ) {
      return;
    }

    previousVisibleRangeRef.current = visibleRange;
    setVisibleRange(visibleRange);
  }, [setVisibleRange, visibleRange]);

  return (
    <div
      ref={scrollRef}
      data-testid="file-list"
      aria-label="List view"
      role="list"
      onScroll={handleScroll}
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
        {renderedItems.map((virtualItem) => {
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
