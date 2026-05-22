import { useState } from 'react';
import type { DirectoryEntry } from '../../api/tauri';
import GlassSurface from '../surfaces/GlassSurface';
import FileGrid from './FileGrid';
import FileList from './FileList';
import DetailsTable from './DetailsTable';
import { useViewportReporting } from '../../hooks/useViewportReporting';
import {
  flattenSearchResultBatches,
  useSearchStore,
  type SearchResultItem,
} from '../../stores/search';

export type ViewMode = 'icon' | 'list' | 'details';

interface FileBrowserProps {
  path: string;
  activeTabId?: string;
  interactionEpoch?: number;
  lastInputAtMs?: number | null;
  entries: DirectoryEntry[];
  loading: boolean;
  error: string | null;
  isTauriRuntime: boolean;
  sortKey?: 'name' | 'modified' | 'size' | 'type';
  sortAscending?: boolean;
  filterKind?: 'all' | 'folders' | 'files' | 'images' | 'documents' | 'videos';
  showHiddenFiles?: boolean;
  showFileExtensions?: boolean;
  onUserInteraction?: () => void;
  onOpenEntry: (entry: DirectoryEntry) => void;
  onOpenSearchResultLocation?: (item: SearchResultItem) => void | Promise<void>;
}

function isFilesystemPath(path: string): boolean {
  return /^[A-Za-z]:\\/.test(path) || path.startsWith('\\\\');
}

const VIEW_BUTTONS: { mode: ViewMode; label: string }[] = [
  { mode: 'list', label: 'List view' },
  { mode: 'icon', label: 'Icon view' },
  { mode: 'details', label: 'Details view' },
];

function FileBrowser({
  path,
  activeTabId = 'tab-1',
  interactionEpoch = 0,
  lastInputAtMs = null,
  entries,
  loading,
  error,
  isTauriRuntime,
  showFileExtensions = true,
  onUserInteraction = () => {},
  onOpenEntry,
  onOpenSearchResultLocation = () => {},
}: FileBrowserProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
  const search = useSearchStore();
  const isPreviewPath = !isFilesystemPath(path);
  const displayEntries = search.query.trim() === '' ? entries : search.visibleEntries;
  const searchResultItems = flattenSearchResultBatches(search.resultBatches);
  useViewportReporting({
    activeTabId,
    interactionEpoch,
    lastInputAtMs,
    visibleRange: null,
    reportWhenRangeMissing: isPreviewPath || loading || Boolean(error) || displayEntries.length === 0,
  });

  function toggleSelect(entryPath: string) {
    onUserInteraction();
    setSelectedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(entryPath)) {
        next.delete(entryPath);
      } else {
        next.add(entryPath);
      }
      return next;
    });
  }

  function handleEntryOpen(entry: DirectoryEntry) {
    if (entry.isFolder) {
      onUserInteraction();
      onOpenEntry(entry);
    }
  }

  function renderView() {
    if (loading) {
      return <div style={{ color: 'var(--text-secondary)', padding: '20px' }}>Loading directory...</div>;
    }
    if (error) {
      return <div role="alert" style={{ color: 'var(--text-error)', padding: '20px' }}>{error}</div>;
    }
    if (isPreviewPath) {
      return (
        <div style={{ color: 'var(--text-secondary)', padding: '20px', textAlign: 'center' }}>
          {isTauriRuntime
            ? 'This location is a virtual entry. Select a drive or folder to browse files.'
            : 'This location is a preview entry. Browse real folders in the desktop app runtime.'}
        </div>
      );
    }
    if (displayEntries.length === 0) {
      return <div style={{ color: 'var(--text-tertiary)', padding: '20px', textAlign: 'center' }}>No items to show.</div>;
    }

    const viewProps = {
      entries: displayEntries,
      selectedPaths,
      showFileExtensions,
      onToggleSelect: toggleSelect,
      onOpenEntry: handleEntryOpen,
      activeTabId,
      interactionEpoch,
      lastInputAtMs,
    };

    switch (viewMode) {
      case 'icon':
        return <FileGrid {...viewProps} />;
      case 'details':
        return <DetailsTable {...viewProps} />;
      case 'list':
      default:
        return <FileList {...viewProps} />;
    }
  }

  return (
    <GlassSurface
      variant="content"
      role="region"
      aria-label="File browser"
      style={{
        display: 'grid',
        gridTemplateRows: 'auto 1fr',
        minHeight: 0,
        overflow: 'hidden',
      }}
    >
      <div style={{ display: 'grid', gap: 12, padding: '12px 16px', borderBottom: '1px solid var(--border-subtle)' }}>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: 12,
          }}
        >
          <div style={{ fontSize: 13, color: 'var(--text-secondary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            <strong style={{ color: 'var(--text-primary)' }}>Location:</strong> {path}
          </div>
          <div style={{ display: 'flex', gap: 4, flexShrink: 0 }}>
            {VIEW_BUTTONS.map(({ mode, label }) => (
              <button
                key={mode}
                type="button"
                role="button"
                aria-label={label}
                aria-pressed={viewMode === mode}
                onClick={() => {
                  onUserInteraction();
                  setViewMode(mode);
                }}
                style={{
                  padding: '4px 10px',
                  fontSize: 11,
                  fontWeight: 500,
                  cursor: 'pointer',
                  border: '1px solid transparent',
                  borderRadius: 'var(--radius-sm)',
                  background: viewMode === mode ? 'var(--surface-floating)' : 'transparent',
                  borderColor: viewMode === mode ? 'var(--border-strong)' : 'transparent',
                  color: viewMode === mode ? 'var(--text-primary)' : 'var(--text-secondary)',
                  transition: 'all 0.2s',
                }}
              >
                {mode.charAt(0).toUpperCase() + mode.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {(search.query.trim() !== '' || search.resultBatches.length > 0) && (
          <section
            aria-label="Search results"
            style={{
              display: 'grid',
              gap: 8,
              padding: 12,
              borderRadius: 'var(--radius-md)',
              background: 'var(--surface-floating)',
              border: '1px solid var(--border-subtle)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
              <div style={{ display: 'grid', gap: 2 }}>
                <strong style={{ fontSize: 13, color: 'var(--text-primary)' }}>
                  {search.recursive ? 'Recursive search' : 'Current folder search'}
                </strong>
                <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>
                  {search.query.trim() === ''
                    ? 'Search cleared.'
                    : `Query "${search.query.trim()}" · ${searchResultItems.length} result(s)`}
                </span>
              </div>
              <span style={{ fontSize: 12, color: 'var(--text-tertiary)' }}>{search.status}</span>
            </div>

            {search.error ? (
              <div role="alert" style={{ color: 'var(--text-error)', fontSize: 12 }}>
                {search.error}
              </div>
            ) : null}

            {searchResultItems.length > 0 ? (
              <div style={{ display: 'grid', gap: 8 }}>
                {searchResultItems.map((item) => (
                  <div
                    key={item.entry.path}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      gap: 12,
                      padding: '8px 10px',
                      borderRadius: 'var(--radius-sm)',
                      background: 'var(--surface-content)',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    <div style={{ display: 'grid', gap: 2, minWidth: 0 }}>
                      <span style={{ color: 'var(--text-primary)', fontSize: 13, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {item.entry.name}
                      </span>
                      <span style={{ color: 'var(--text-tertiary)', fontSize: 12, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {item.entry.path}
                      </span>
                    </div>
                    <button
                      type="button"
                      aria-label={`Open location for ${item.entry.name}`}
                      onClick={() => void onOpenSearchResultLocation(item)}
                    >
                      Open location
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <div style={{ color: 'var(--text-tertiary)', fontSize: 12 }}>No search results.</div>
            )}
          </section>
        )}
      </div>
      <div style={{ minHeight: 0, height: '100%' }}>{renderView()}</div>
    </GlassSurface>
  );
}

export default FileBrowser;
