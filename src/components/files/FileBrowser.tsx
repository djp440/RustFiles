import { useCallback, useEffect, useRef, useState } from 'react';
import type { DirectoryEntry, SystemActionFeedback } from '../../api/tauri';
import { createDragOperation, dropDragOperation } from '../../api/tauri';
import GlassSurface from '../surfaces/GlassSurface';
import FileGrid from './FileGrid';
import FileList from './FileList';
import DetailsTable from './DetailsTable';
import InlineRename from './InlineRename';
import DeleteConfirmDialog from '../dialogs/DeleteConfirmDialog';
import PropertiesDialog from '../dialogs/PropertiesDialog';
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
  onCreateFolder?: (name: string) => Promise<string | null>;
  onRenameItem?: (path: string, newName: string) => Promise<string | null>;
  onDeleteToRecycleBin?: (path: string) => Promise<string | null>;
  onDeletePermanently?: (path: string, confirmationToken: string) => Promise<string | null>;
  onOpenWithDefaultApp?: (path: string) => Promise<SystemActionFeedback>;
  onOpenTerminal?: (path: string) => Promise<SystemActionFeedback>;
  onShowProperties?: (path: string) => Promise<SystemActionFeedback>;
}

function isFilesystemPath(path: string): boolean {
  return /^[A-Za-z]:\\/.test(path) || path.startsWith('\\\\');
}

const VIEW_BUTTONS: { mode: ViewMode; label: string }[] = [
  { mode: 'list', label: 'List view' },
  { mode: 'icon', label: 'Icon view' },
  { mode: 'details', label: 'Details view' },
];

const actionButtonStyle: React.CSSProperties = {
  padding: '3px 10px',
  fontSize: 12,
  cursor: 'pointer',
  border: '1px solid var(--border-subtle)',
  borderRadius: 'var(--radius-sm)',
  background: 'var(--surface-content)',
  color: 'var(--text-secondary)',
  whiteSpace: 'nowrap',
};

function NewFolderInput({ onCommit, onCancel }: { onCommit: (name: string) => void; onCancel: () => void }) {
  const [value, setValue] = useState('新建文件夹');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter') {
      onCommit(value);
    } else if (e.key === 'Escape') {
      onCancel();
    }
  }

  return (
    <div style={{ display: 'flex', gap: 8, alignItems: 'center', flex: 1 }}>
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
        aria-label="New folder name"
        style={{
          background: 'var(--surface-content)',
          color: 'var(--text-primary)',
          border: '1px solid var(--border-strong)',
          borderRadius: 'var(--radius-sm)',
          padding: '3px 8px',
          fontSize: 13,
          flex: 1,
          outline: 'none',
        }}
      />
      <button
        type="button"
        aria-label="Create"
        onClick={() => onCommit(value)}
        style={{
          padding: '4px 12px',
          fontSize: 13,
          cursor: 'pointer',
          border: '1px solid var(--border-strong)',
          borderRadius: 'var(--radius-sm)',
          background: 'var(--surface-floating)',
          color: 'var(--text-primary)',
        }}
      >
        创建
      </button>
      <button
        type="button"
        onClick={onCancel}
        style={{
          padding: '4px 12px',
          fontSize: 13,
          cursor: 'pointer',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-sm)',
          background: 'transparent',
          color: 'var(--text-secondary)',
        }}
      >
        取消
      </button>
    </div>
  );
}

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
  onCreateFolder = async () => null,
  onRenameItem = async () => null,
  onDeleteToRecycleBin = async () => null,
  onDeletePermanently = async () => null,
  onOpenWithDefaultApp = async () => ({ kind: 'open', path: '', preview: true, error: '未实现' }),
  onOpenTerminal = async () => ({ kind: 'terminal', path: '', preview: true, error: '未实现' }),
  onShowProperties = async () => ({ kind: 'properties', path: '', preview: true, error: '未实现' }),
}: FileBrowserProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
  const [renamingPath, setRenamingPath] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<DirectoryEntry | null>(null);
  const [propertiesTarget, setPropertiesTarget] = useState<DirectoryEntry | null>(null);
  const [newFolderInput, setNewFolderInput] = useState(false);
  const [systemFeedback, setSystemFeedback] = useState<string | null>(null);
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

  const getSelectedEntry = useCallback((): DirectoryEntry | null => {
    if (selectedPaths.size === 0) return null;
    const firstPath = [...selectedPaths][0];
    return displayEntries.find((e) => e.path === firstPath) ?? null;
  }, [selectedPaths, displayEntries]);

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

  function handleRenameCommit(newName: string) {
    const target = renamingPath;
    if (!target) return;
    setRenamingPath(null);
    void onRenameItem(target, newName);
  }

  function handleRenameCancel() {
    setRenamingPath(null);
  }

  function handleDeleteRecycleBin() {
    const target = deleteTarget;
    if (!target) return;
    setDeleteTarget(null);
    void onDeleteToRecycleBin(target.path);
  }

  function handleDeletePermanently(confirmationToken: string) {
    const target = deleteTarget;
    if (!target) return;
    setDeleteTarget(null);
    void onDeletePermanently(target.path, confirmationToken);
  }

  function handleDeleteCancel() {
    setDeleteTarget(null);
  }

  function handlePropertiesClose() {
    setPropertiesTarget(null);
  }

  async function handleSystemProperties() {
    const target = propertiesTarget;
    if (!target) return;
    const feedback = await onShowProperties(target.path);
    if (feedback.error) {
      setSystemFeedback(feedback.error);
    }
  }

  async function handleActionOpen() {
    const entry = getSelectedEntry();
    if (!entry) return;
    onUserInteraction();
    const feedback = await onOpenWithDefaultApp(entry.path);
    if (feedback.error) {
      setSystemFeedback(feedback.preview ? `预览模式：${feedback.error}` : feedback.error);
    } else {
      setSystemFeedback(null);
    }
  }

  async function handleActionTerminal() {
    onUserInteraction();
    const feedback = await onOpenTerminal(path);
    if (feedback.error) {
      setSystemFeedback(feedback.preview ? `预览模式：${feedback.error}` : feedback.error);
    } else {
      setSystemFeedback(null);
    }
  }

  async function handleActionProperties() {
    const entry = getSelectedEntry();
    if (!entry) return;
    onUserInteraction();
    setPropertiesTarget(entry);
  }

  function handleActionRename() {
    const entry = getSelectedEntry();
    if (!entry) return;
    onUserInteraction();
    setRenamingPath(entry.path);
  }

  function handleActionDelete() {
    const entry = getSelectedEntry();
    if (!entry) return;
    onUserInteraction();
    setDeleteTarget(entry);
  }

  async function handleCreateFolder(name: string) {
    const trimmed = name.trim();
    if (trimmed.length === 0) return;
    setNewFolderInput(false);
    await onCreateFolder(trimmed);
  }

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (renamingPath || deleteTarget || propertiesTarget || newFolderInput) return;

      if (e.key === 'F2' && selectedPaths.size > 0) {
        e.preventDefault();
        const entry = getSelectedEntry();
        if (entry) {
          onUserInteraction();
          setRenamingPath(entry.path);
        }
      } else if (e.key === 'Delete' && selectedPaths.size > 0) {
        e.preventDefault();
        const entry = getSelectedEntry();
        if (entry) {
          onUserInteraction();
          setDeleteTarget(entry);
        }
      }
    },
    [renamingPath, deleteTarget, propertiesTarget, newFolderInput, selectedPaths, getSelectedEntry, onUserInteraction],
  );

  const renderView = useCallback(() => {
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
  }, [loading, error, isPreviewPath, isTauriRuntime, displayEntries, selectedPaths, showFileExtensions, viewMode, activeTabId, interactionEpoch, lastInputAtMs]);

  const selectedCount = selectedPaths.size;
  const hasSelection = !isPreviewPath && selectedCount > 0;

  return (
    <div onKeyDown={handleKeyDown} style={{ outline: 'none' }} tabIndex={-1}>
      <GlassSurface
        variant="content"
        role="region"
        aria-label="File browser"
        style={{
          display: 'grid',
          gridTemplateRows: 'auto auto 1fr',
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

          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              flexWrap: 'wrap',
            }}
          >
            <button
              type="button"
              aria-label="New Folder"
              onClick={() => setNewFolderInput(true)}
              disabled={isPreviewPath}
              style={actionButtonStyle}
            >
              + 新建文件夹
            </button>
            <button
              type="button"
              aria-label="Rename"
              onClick={handleActionRename}
              disabled={!hasSelection}
              style={actionButtonStyle}
            >
              重命名
            </button>
            <button
              type="button"
              aria-label="Delete"
              onClick={handleActionDelete}
              disabled={!hasSelection}
              style={actionButtonStyle}
            >
              删除
            </button>
            <span style={{ width: 8, flexShrink: 0 }} />
            <button
              type="button"
              aria-label="Open"
              onClick={() => void handleActionOpen()}
              disabled={!hasSelection}
              style={actionButtonStyle}
            >
              打开
            </button>
            <button
              type="button"
              aria-label="Terminal"
              onClick={() => void handleActionTerminal()}
              style={actionButtonStyle}
            >
              终端
            </button>
            <button
              type="button"
              aria-label="Properties"
              onClick={() => void handleActionProperties()}
              disabled={!hasSelection}
              style={actionButtonStyle}
            >
              属性
            </button>
          </div>

          {newFolderInput && (
            <div
              role="dialog"
              aria-label="New Folder"
              style={{
                display: 'flex',
                gap: 8,
                alignItems: 'center',
                padding: '8px 12px',
                borderRadius: 'var(--radius-md)',
                background: 'var(--surface-floating)',
                border: '1px solid var(--border-strong)',
              }}
            >
              <span style={{ fontSize: 13, color: 'var(--text-secondary)' }}>文件夹名：</span>
              <NewFolderInput
                onCommit={handleCreateFolder}
                onCancel={() => setNewFolderInput(false)}
              />
            </div>
          )}

          {renamingPath && (() => {
            const entry = displayEntries.find((e) => e.path === renamingPath);
            if (!entry) return null;
            const siblingNames = displayEntries
              .filter((e) => e.path !== renamingPath)
              .map((e) => e.name);
            return (
              <div
                style={{
                  display: 'flex',
                  gap: 8,
                  alignItems: 'center',
                  padding: '8px 12px',
                  borderRadius: 'var(--radius-md)',
                  background: 'var(--surface-floating)',
                  border: '1px solid var(--border-strong)',
                }}
              >
                <span style={{ fontSize: 13, color: 'var(--text-secondary)', whiteSpace: 'nowrap' }}>
                  重命名：
                </span>
                <InlineRename
                  initialName={entry.name}
                  existingNames={siblingNames}
                  isFolder={entry.isFolder}
                  onCommit={handleRenameCommit}
                  onCancel={handleRenameCancel}
                />
              </div>
            );
          })()}

          {systemFeedback && (
            <div
              role="alert"
              style={{
                color: 'var(--text-secondary)',
                fontSize: 12,
                padding: '4px 8px',
                borderRadius: 'var(--radius-sm)',
                background: 'var(--surface-floating)',
                border: '1px solid var(--border-subtle)',
              }}
            >
              {systemFeedback}
            </div>
          )}

          {(search.query.trim() !== '' || search.resultBatches.length > 0) && (
            <section aria-label="Search results" style={{ display: 'grid', gap: 8, padding: 12, borderRadius: 'var(--radius-md)', background: 'var(--surface-floating)', border: '1px solid var(--border-subtle)' }}>
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
                    <div key={item.entry.path} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, padding: '8px 10px', borderRadius: 'var(--radius-sm)', background: 'var(--surface-content)', border: '1px solid var(--border-subtle)' }}>
                      <div style={{ display: 'grid', gap: 2, minWidth: 0 }}>
                        <span style={{ color: 'var(--text-primary)', fontSize: 13, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                          {item.entry.name}
                        </span>
                        <span style={{ color: 'var(--text-tertiary)', fontSize: 12, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                          {item.entry.path}
                        </span>
                      </div>
                      <button type="button" aria-label={`Open location for ${item.entry.name}`} onClick={() => void onOpenSearchResultLocation(item)}>
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
        <div
          style={{ minHeight: 0, height: '100%' }}
          onDragOver={(e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = e.ctrlKey || e.metaKey ? 'copy' : 'move';
          }}
          onDrop={async (e) => {
            e.preventDefault();
            const raw = e.dataTransfer.getData('text/x-rustfiles-drag-paths');
            if (!raw) return;
            try {
              const sourcePaths: string[] = JSON.parse(raw);
              if (sourcePaths.length === 0) return;
              const requestedType = e.ctrlKey || e.metaKey ? 'copy' : undefined;
              const opId = await createDragOperation(sourcePaths, requestedType ?? 'move', activeTabId);
              await dropDragOperation(opId, path, requestedType);
            } catch {
              // ignore drop errors in preview mode
            }
          }}
        >{renderView()}</div>
      </GlassSurface>
      {deleteTarget ? (
        <DeleteConfirmDialog
          itemName={deleteTarget.name}
          onRecycleBin={handleDeleteRecycleBin}
          onPermanentDelete={handleDeletePermanently}
          onCancel={handleDeleteCancel}
        />
      ) : null}
      {propertiesTarget ? (
        <PropertiesDialog
          entry={propertiesTarget}
          isTauriRuntime={isTauriRuntime}
          onClose={handlePropertiesClose}
          onSystemProperties={() => void handleSystemProperties()}
        />
      ) : null}
    </div>
  );
}

export default FileBrowser;
