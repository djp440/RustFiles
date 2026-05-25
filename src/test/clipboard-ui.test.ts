import { describe, it, expect, beforeEach } from 'vitest';
import { selectionStore, resetSelectionStore } from '../stores/selection';

describe('selectionStore', () => {
  beforeEach(() => {
    resetSelectionStore();
  });

  it('可以存储和清除选中路径', () => {
    selectionStore.setSelectedPaths(['/path/a', '/path/b']);
    expect(selectionStore.selectedPaths).toEqual(['/path/a', '/path/b']);

    selectionStore.clearSelection();
    expect(selectionStore.selectedPaths).toEqual([]);
  });

  it('可设置 clipboard copy 操作', () => {
    selectionStore.setClipboardCopy(['/path/a', '/path/b']);
    expect(selectionStore.clipboardOp).not.toBeNull();
    expect(selectionStore.clipboardOp!.type).toBe('copy');
    expect(selectionStore.clipboardOp!.paths).toEqual(['/path/a', '/path/b']);
    expect(selectionStore.clipboardOp!.operationId).toBeTruthy();
  });

  it('可设置 clipboard cut 操作', () => {
    selectionStore.setClipboardCut(['/path/x']);
    expect(selectionStore.clipboardOp).not.toBeNull();
    expect(selectionStore.clipboardOp!.type).toBe('cut');
    expect(selectionStore.clipboardOp!.paths).toEqual(['/path/x']);
  });

  it('cut 状态会标记哪些路径是 cut-pending', () => {
    selectionStore.setClipboardCut(['/path/pending1', '/path/pending2']);
    expect(selectionStore.isCutPending('/path/pending1')).toBe(true);
    expect(selectionStore.isCutPending('/path/pending2')).toBe(true);
    expect(selectionStore.isCutPending('/path/not-pending')).toBe(false);
  });

  it('clipboard 被清除后 isCutPending 返回 false', () => {
    selectionStore.setClipboardCut(['/path/pending']);
    expect(selectionStore.isCutPending('/path/pending')).toBe(true);

    selectionStore.clearClipboard();
    expect(selectionStore.clipboardOp).toBeNull();
    expect(selectionStore.isCutPending('/path/pending')).toBe(false);
  });

  it('setClipboardCopy 会设置 type 为 copy', () => {
    selectionStore.setClipboardCopy(['/path/doc']);
    expect(selectionStore.clipboardOp!.type).toBe('copy');
  });

  it('setClipboardCut 会设置 type 为 cut', () => {
    selectionStore.setClipboardCut(['/path/img']);
    expect(selectionStore.clipboardOp!.type).toBe('cut');
  });

  it('clearClipboard 清除所有 clipboard 状态', () => {
    selectionStore.setClipboardCopy(['/path/a']);
    expect(selectionStore.clipboardOp).not.toBeNull();

    selectionStore.clearClipboard();
    expect(selectionStore.clipboardOp).toBeNull();
    expect(selectionStore.selectedPaths).toEqual([]);
  });
});
