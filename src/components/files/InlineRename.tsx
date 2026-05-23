import { useEffect, useRef, useState } from 'react';

const RESERVED_NAMES = new Set([
  'CON', 'PRN', 'AUX', 'NUL',
  'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
  'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
]);

const ILLEGAL_CHARS_RE = /[<>:"/\\|?*]/;

interface InlineRenameProps {
  initialName: string;
  existingNames: string[];
  isFolder: boolean;
  onCommit: (newName: string) => void;
  onCancel: () => void;
}

function extractStem(name: string): string {
  const lastDot = name.lastIndexOf('.');
  if (lastDot === -1 || lastDot === 0) {
    return name;
  }
  return name.substring(0, lastDot);
}

function validateName(value: string, existingNames: string[], originalName: string): string | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return '文件名不能为空';
  }

  if (ILLEGAL_CHARS_RE.test(trimmed)) {
    return '文件名包含非法字符 ( < > : " / \\ | ? * )';
  }

  const upperName = trimmed.toUpperCase();
  const stem = extractStem(trimmed).toUpperCase();

  if (RESERVED_NAMES.has(upperName) || RESERVED_NAMES.has(stem)) {
    return '此名称是 Windows 保留名，无法使用';
  }

  if (trimmed !== originalName && existingNames.some((n) => n.toLowerCase() === trimmed.toLowerCase())) {
    return '当前目录已存在同名项目';
  }

  if (trimmed.endsWith('.') || trimmed.endsWith(' ')) {
    return '文件名不能以空格或句点结尾';
  }

  return null;
}

function InlineRename({
  initialName,
  existingNames,
  isFolder: _isFolder,
  onCommit,
  onCancel,
}: InlineRenameProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [value, setValue] = useState(initialName);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
    setValue(e.target.value);
    setError(null);
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter') {
      e.preventDefault();
      const validationError = validateName(value, existingNames, initialName);
      if (validationError) {
        setError(validationError);
        return;
      }
      onCommit(value.trim());
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onCancel();
    }
  }

  function handleBlur() {
    onCancel();
  }

  return (
    <div
      style={{
        display: 'inline-flex',
        flexDirection: 'column',
        gap: 2,
        minWidth: 0,
      }}
    >
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        onBlur={handleBlur}
        aria-label="Rename input"
        style={{
          background: 'var(--surface-floating)',
          color: 'var(--text-primary)',
          border: error ? '1px solid var(--text-error)' : '1px solid var(--border-strong)',
          borderRadius: 'var(--radius-sm)',
          padding: '2px 6px',
          fontSize: 13,
          outline: 'none',
          minWidth: 0,
          width: '100%',
          boxSizing: 'border-box',
        }}
      />
      {error ? (
        <span
          role="alert"
          style={{
            color: 'var(--text-error)',
            fontSize: 11,
            whiteSpace: 'nowrap',
          }}
        >
          {error}
        </span>
      ) : null}
    </div>
  );
}

export default InlineRename;
