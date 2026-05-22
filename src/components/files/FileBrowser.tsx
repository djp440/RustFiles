import type { DirectoryEntry } from '../../api/tauri';

interface FileBrowserProps {
  path: string;
  entries: DirectoryEntry[];
  loading: boolean;
  error: string | null;
  onOpenEntry: (entry: DirectoryEntry) => void;
}

function FileBrowser({ path, entries, loading, error, onOpenEntry }: FileBrowserProps) {
  return (
    <section aria-label="File browser" style={{ padding: 16, display: 'grid', gap: 12 }}>
      <div>
        <strong>Current location:</strong> {path}
      </div>
      {loading ? <div>Loading directory…</div> : null}
      {error ? <div role="alert">{error}</div> : null}
      {!loading && !error && entries.length === 0 ? <div>No items to show.</div> : null}
      <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'grid', gap: 8 }}>
        {entries.map((entry) => (
          <li key={entry.path}>
            <button
              type="button"
              onClick={() => onOpenEntry(entry)}
              disabled={!entry.isFolder}
              style={{ width: '100%', textAlign: 'left' }}
            >
              <span>{entry.name}</span>
              <span style={{ marginLeft: 8, opacity: 0.7 }}>
                {entry.isFolder ? 'Folder' : 'File'}
              </span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

export default FileBrowser;
