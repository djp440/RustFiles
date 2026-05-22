import { useState } from 'react';
import type { DirectoryEntry } from '../../api/tauri';
import GlassSurface from '../surfaces/GlassSurface';
import FileGrid from './FileGrid';
import FileList from './FileList';
import DetailsTable from './DetailsTable';

export type ViewMode = 'icon' | 'list' | 'details';

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

const VIEW_BUTTONS: { mode: ViewMode; label: string }[] = [
  { mode: 'list', label: 'List view' },
  { mode: 'icon', label: 'Icon view' },
  { mode: 'details', label: 'Details view' },
];

function FileBrowser({
  path,
  entries,
  loading,
  error,
  isTauriRuntime,
  showFileExtensions = true,
  onOpenEntry,
}: FileBrowserProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
  const isPreviewPath = !isFilesystemPath(path);

  function toggleSelect(entryPath: string) {
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
    if (entries.length === 0) {
      return <div style={{ color: 'var(--text-tertiary)', padding: '20px', textAlign: 'center' }}>No items to show.</div>;
    }

    const viewProps = {
      entries,
      selectedPaths,
      showFileExtensions,
      onToggleSelect: toggleSelect,
      onOpenEntry: handleEntryOpen,
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
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '12px 16px',
          borderBottom: '1px solid var(--border-subtle)',
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
              onClick={() => setViewMode(mode)}
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
      <div style={{ overflow: 'auto', minHeight: 0 }}>
        {renderView()}
      </div>
    </GlassSurface>
  );
}

export default FileBrowser;
