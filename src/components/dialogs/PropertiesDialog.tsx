import { useCallback, useEffect, useRef } from 'react';
import type { DirectoryEntry } from '../../api/tauri';

interface PropertiesDialogProps {
  entry: DirectoryEntry;
  isTauriRuntime: boolean;
  onClose: () => void;
  onSystemProperties: () => void;
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let unitIndex = 0;
  let size = bytes;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

function formatDate(epochSeconds: number): string {
  if (epochSeconds <= 0) return 'Unknown';
  const date = new Date(epochSeconds * 1000);
  return date.toLocaleString();
}

function PropertiesDialog({
  entry,
  isTauriRuntime,
  onClose,
  onSystemProperties,
}: PropertiesDialogProps) {
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    closeRef.current?.focus();
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [onClose],
  );

  return (
    <div
      role="dialog"
      aria-label="Properties"
      onKeyDown={handleKeyDown}
      style={{
        position: 'fixed',
        inset: 0,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'rgba(0, 0, 0, 0.5)',
        zIndex: 1000,
      }}
    >
      <div
        style={{
          background: 'var(--surface-floating)',
          border: '1px solid var(--border-strong)',
          borderRadius: 'var(--radius-md)',
          padding: 24,
          minWidth: 360,
          maxWidth: 480,
          display: 'grid',
          gap: 16,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <h3 style={{ margin: 0, fontSize: 16, color: 'var(--text-primary)' }}>
            属性
          </h3>
          <span
            style={{
              fontSize: 12,
              color: 'var(--text-tertiary)',
              padding: '2px 8px',
              borderRadius: 'var(--radius-sm)',
              background: 'var(--surface-content)',
            }}
          >
            {entry.isFolder ? '文件夹' : '文件'}
          </span>
        </div>

        <div style={{ display: 'grid', gap: 10 }}>
          <PropertyRow label="名称" value={entry.name} />
          <PropertyRow label="路径" value={entry.path} />
          <PropertyRow label="类型" value={entry.isFolder ? '文件夹' : getFileType(entry.name)} />
          <PropertyRow label="大小" value={entry.isFolder ? '--' : formatFileSize(entry.size)} />
          <PropertyRow label="修改时间" value={formatDate(entry.modified)} />
        </div>

        {isTauriRuntime ? (
          <button
            type="button"
            onClick={onSystemProperties}
            style={{
              padding: '6px 16px',
              fontSize: 13,
              cursor: 'pointer',
              border: '1px solid var(--border-strong)',
              borderRadius: 'var(--radius-sm)',
              background: 'var(--surface-content)',
              color: 'var(--text-primary)',
              justifySelf: 'start',
            }}
          >
            系统属性
          </button>
        ) : (
          <p style={{ margin: 0, color: 'var(--text-tertiary)', fontSize: 12 }}>
            浏览器预览模式下仅显示基本信息。桌面运行时支持完整系统属性。
          </p>
        )}

        <button
          type="button"
          ref={closeRef}
          onClick={onClose}
          aria-label="Close"
          style={{
            padding: '6px 16px',
            fontSize: 13,
            cursor: 'pointer',
            border: '1px solid var(--border-strong)',
            borderRadius: 'var(--radius-sm)',
            background: 'var(--surface-content)',
            color: 'var(--text-primary)',
            justifySelf: 'end',
          }}
        >
          关闭
        </button>
      </div>
    </div>
  );
}

function PropertyRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: '80px 1fr', gap: 8, alignItems: 'start' }}>
      <span style={{ color: 'var(--text-tertiary)', fontSize: 12 }}>{label}</span>
      <span
        style={{
          color: 'var(--text-primary)',
          fontSize: 13,
          wordBreak: 'break-all',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
        }}
      >
        {value}
      </span>
    </div>
  );
}

function getFileType(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase();
  if (!ext || ext === filename) return '未知类型';
  const typeMap: Record<string, string> = {
    txt: '文本文档',
    md: 'Markdown 文档',
    js: 'JavaScript 文件',
    ts: 'TypeScript 文件',
    tsx: 'TypeScript React 文件',
    json: 'JSON 文件',
    css: 'CSS 样式表',
    html: 'HTML 文档',
    png: 'PNG 图像',
    jpg: 'JPEG 图像',
    jpeg: 'JPEG 图像',
    gif: 'GIF 图像',
    svg: 'SVG 图像',
    pdf: 'PDF 文档',
    zip: 'ZIP 压缩包',
    exe: '可执行文件',
    dll: '动态链接库',
  };
  return typeMap[ext] ?? `.${ext} 文件`;
}

export default PropertiesDialog;
