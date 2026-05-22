import { useEffect, useState, type FormEvent } from 'react';
import Breadcrumb from './Breadcrumb';
import type { BreadcrumbSegment } from '../../stores/tabs';

interface NavigationBarProps {
  path: string;
  canGoBack: boolean;
  canGoForward: boolean;
  breadcrumbSegments: BreadcrumbSegment[];
  onBack: () => void;
  onForward: () => void;
  onSubmitPath: (path: string) => void;
  onSelectBreadcrumb: (path: string) => void;
}

function NavigationBar({
  path,
  canGoBack,
  canGoForward,
  breadcrumbSegments,
  onBack,
  onForward,
  onSubmitPath,
  onSelectBreadcrumb,
}: NavigationBarProps) {
  const [draftPath, setDraftPath] = useState(path);

  useEffect(() => {
    setDraftPath(path);
  }, [path]);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    onSubmitPath(draftPath);
  }

  return (
    <section
      aria-label="Navigation"
      style={{
        display: 'grid',
        gridTemplateColumns: 'auto auto 1fr',
        gap: 12,
        padding: 16,
        borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
      }}
    >
      <button type="button" onClick={onBack} disabled={!canGoBack}>
        Back
      </button>
      <button type="button" onClick={onForward} disabled={!canGoForward}>
        Forward
      </button>
      <div style={{ display: 'grid', gap: 8 }}>
        <form onSubmit={handleSubmit}>
          <label style={{ display: 'grid', gap: 4 }}>
            <span style={{ fontSize: 12 }}>Path</span>
            <input
              aria-label="Path"
              value={draftPath}
              onChange={(event) => setDraftPath(event.target.value)}
            />
          </label>
        </form>
        <Breadcrumb segments={breadcrumbSegments} onSelect={onSelectBreadcrumb} />
      </div>
    </section>
  );
}

export default NavigationBar;
