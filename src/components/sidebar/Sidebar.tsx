import type { DriveInfo, SidebarRoots as SidebarRootsData } from '../../api/tauri';

interface SidebarProps {
  roots: SidebarRootsData;
  drives: DriveInfo[];
  activePath: string;
  onSelectPath: (path: string) => void;
}

const rootLabels: Array<{ key: keyof SidebarRootsData; label: string }> = [
  { key: 'thisPc', label: 'This PC' },
  { key: 'desktop', label: 'Desktop' },
  { key: 'downloads', label: 'Downloads' },
  { key: 'documents', label: 'Documents' },
  { key: 'pictures', label: 'Pictures' },
  { key: 'videos', label: 'Videos' },
  { key: 'music', label: 'Music' },
];

function Sidebar({ roots, drives, activePath, onSelectPath }: SidebarProps) {
  return (
    <aside
      aria-label="Sidebar"
      style={{
        width: 220,
        padding: 16,
        borderRight: '1px solid rgba(255, 255, 255, 0.08)',
        display: 'grid',
        gap: 16,
      }}
    >
      <section>
        <h2 style={{ fontSize: 14, margin: '0 0 8px' }}>Favorites</h2>
        <div style={{ display: 'grid', gap: 8 }}>
          {rootLabels.map(({ key, label }) => {
            const path = roots[key];

            return (
              <button
                key={key}
                type="button"
                onClick={() => onSelectPath(path)}
                aria-pressed={activePath === path}
                style={{ textAlign: 'left' }}
              >
                {label}
              </button>
            );
          })}
        </div>
      </section>
      <section>
        <h2 style={{ fontSize: 14, margin: '0 0 8px' }}>Drives</h2>
        <div style={{ display: 'grid', gap: 8 }}>
          {drives.length > 0 ? (
            drives.map((drive) => (
              <button
                key={drive.path}
                type="button"
                onClick={() => onSelectPath(drive.path)}
                aria-pressed={activePath === drive.path}
                style={{ textAlign: 'left' }}
              >
                {drive.name || drive.path}
              </button>
            ))
          ) : (
            <span style={{ fontSize: 12, opacity: 0.7 }}>No drives available</span>
          )}
        </div>
      </section>
    </aside>
  );
}

export default Sidebar;
