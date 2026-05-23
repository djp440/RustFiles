import { useEffect, useRef, useState } from 'react';

interface DeleteConfirmDialogProps {
  itemName: string;
  onRecycleBin: () => void;
  onPermanentDelete: (confirmationToken: string) => void;
  onCancel: () => void;
}

function DeleteConfirmDialog({
  itemName,
  onRecycleBin,
  onPermanentDelete,
  onCancel,
}: DeleteConfirmDialogProps) {
  const cancelRef = useRef<HTMLButtonElement>(null);
  const [permanentPhase, setPermanentPhase] = useState(false);

  useEffect(() => {
    cancelRef.current?.focus();
  }, []);

  useEffect(() => {
    if (permanentPhase) {
      cancelRef.current?.focus();
    }
  }, [permanentPhase]);

  function handleRecycleBin() {
    onRecycleBin();
  }

  function handlePermanentClick() {
    setPermanentPhase(true);
  }

  function handleConfirmPermanent() {
    onPermanentDelete(`confirm-${Date.now()}`);
  }

  function handleBackFromPermanent() {
    setPermanentPhase(false);
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Escape') {
      if (permanentPhase) {
        setPermanentPhase(false);
      } else {
        onCancel();
      }
    }
  }

  return (
    <div
      role="dialog"
      aria-label="Delete confirmation"
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
        <h3 style={{ margin: 0, fontSize: 16, color: 'var(--text-primary)' }}>
          {permanentPhase ? '确认永久删除' : '删除确认'}
        </h3>

        {permanentPhase ? (
          <div style={{ display: 'grid', gap: 8 }}>
            <p style={{ margin: 0, color: 'var(--text-secondary)', fontSize: 13 }}>
              此操作不可撤销。<strong style={{ color: 'var(--text-error)' }}>"{itemName}"</strong> 将被永久删除，无法从回收站恢复。
            </p>
            <p style={{ margin: 0, color: 'var(--text-error)', fontSize: 12, fontWeight: 500 }}>
              确认要继续吗？
            </p>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button
                type="button"
                ref={cancelRef}
                onClick={handleBackFromPermanent}
                style={{
                  padding: '6px 16px',
                  fontSize: 13,
                  cursor: 'pointer',
                  border: '1px solid var(--border-strong)',
                  borderRadius: 'var(--radius-sm)',
                  background: 'var(--surface-content)',
                  color: 'var(--text-primary)',
                }}
              >
                返回
              </button>
              <button
                type="button"
                onClick={handleConfirmPermanent}
                style={{
                  padding: '6px 16px',
                  fontSize: 13,
                  cursor: 'pointer',
                  border: '1px solid var(--text-error)',
                  borderRadius: 'var(--radius-sm)',
                  background: 'var(--text-error)',
                  color: '#fff',
                  fontWeight: 600,
                }}
              >
                确认永久删除
              </button>
            </div>
          </div>
        ) : (
          <div style={{ display: 'grid', gap: 8 }}>
            <p style={{ margin: 0, color: 'var(--text-secondary)', fontSize: 13 }}>
              选择对 <strong>"{itemName}"</strong> 的删除方式：
            </p>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', flexWrap: 'wrap' }}>
              <button
                type="button"
                ref={cancelRef}
                onClick={onCancel}
                style={{
                  padding: '6px 16px',
                  fontSize: 13,
                  cursor: 'pointer',
                  border: '1px solid var(--border-strong)',
                  borderRadius: 'var(--radius-sm)',
                  background: 'var(--surface-content)',
                  color: 'var(--text-primary)',
                }}
              >
                取消
              </button>
              <button
                type="button"
                onClick={handlePermanentClick}
                style={{
                  padding: '6px 16px',
                  fontSize: 13,
                  cursor: 'pointer',
                  border: '1px solid var(--text-error)',
                  borderRadius: 'var(--radius-sm)',
                  background: 'transparent',
                  color: 'var(--text-error)',
                }}
              >
                永久删除
              </button>
              <button
                type="button"
                onClick={handleRecycleBin}
                style={{
                  padding: '6px 16px',
                  fontSize: 13,
                  cursor: 'pointer',
                  border: '1px solid var(--border-strong)',
                  borderRadius: 'var(--radius-sm)',
                  background: 'var(--surface-floating)',
                  color: 'var(--text-primary)',
                }}
              >
                移到回收站
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default DeleteConfirmDialog;
