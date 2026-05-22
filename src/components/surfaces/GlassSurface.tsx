import React from 'react';

export type GlassVariant = 'chrome' | 'content' | 'floating' | 'solid-safety';

interface GlassSurfaceProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: GlassVariant;
  children?: React.ReactNode;
}

const GlassSurface = React.forwardRef<HTMLDivElement, GlassSurfaceProps>(
  ({ variant = 'content', children, style, ...props }, ref) => {
    const getSurfaceStyles = (v: GlassVariant): React.CSSProperties => {
      switch (v) {
        case 'chrome':
          return {
            background: 'var(--surface-chrome)',
            backdropFilter: 'blur(var(--glass-blur-lg))',
            borderBottom: '1px solid var(--glass-border-light)',
          };
        case 'floating':
          return {
            background: 'var(--surface-floating)',
            backdropFilter: 'blur(var(--glass-blur-sm))',
            border: '1px solid var(--glass-border-strong)',
            borderRadius: 'var(--radius-lg)',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4)',
          };
        case 'solid-safety':
          return {
            background: 'var(--surface-solid-safety)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
          };
        case 'content':
        default:
          return {
            background: 'var(--surface-content)',
            backdropFilter: 'blur(var(--glass-blur-md))',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
          };
      }
    };

    const baseStyle: React.CSSProperties = {
      ...getSurfaceStyles(variant),
      ...style,
    };

    return (
      <div
        ref={ref}
        data-surface-variant={variant}
        style={baseStyle}
        {...props}
      >
        {children}
      </div>
    );
  }
);

GlassSurface.displayName = 'GlassSurface';

export default GlassSurface;
