import type { BreadcrumbSegment } from '../../stores/tabs';

interface BreadcrumbProps {
  segments: BreadcrumbSegment[];
  onSelect: (path: string) => void;
}

function Breadcrumb({ segments, onSelect }: BreadcrumbProps) {
  return (
    <nav aria-label="Breadcrumb" style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
      {segments.map((segment, index) => (
        <button
          key={`${segment.path}-${index}`}
          type="button"
          onClick={() => onSelect(segment.path)}
          style={{
            border: 'none',
            background: 'transparent',
            color: 'inherit',
            cursor: 'pointer',
            padding: 0,
          }}
        >
          {index > 0 ? ' / ' : ''}
          {segment.label}
        </button>
      ))}
    </nav>
  );
}

export default Breadcrumb;
