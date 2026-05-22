import { type ReactElement } from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import FileBrowser from '../components/files/FileBrowser';

function generateEntries(count: number) {
  const entries = [];
  for (let i = 0; i < count; i++) {
    entries.push({
      path: `C:\\test\\file_${i}.txt`,
      name: `file_${i}.txt`,
      size: i * 100,
      modified: 1000 + i,
      isHidden: false,
      isFolder: false,
    });
  }
  return entries;
}

function renderWithHeight(ui: ReactElement, height = 800) {
  return render(
    <div style={{ height: `${height}px` }} data-testid="test-wrapper">
      {ui}
    </div>
  );
}

describe('VirtualList', () => {
  it('renders data-testid for each view mode', () => {
    const entries = generateEntries(10);

    renderWithHeight(
      <FileBrowser
        path="C:\\test"
        entries={entries}
        loading={false}
        error={null}
        isTauriRuntime={true}
        onOpenEntry={vi.fn()}
      />
    );

    expect(screen.getByTestId('file-list')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /icon view/i }));
    expect(screen.getByTestId('file-grid')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /details view/i }));
    expect(screen.getByTestId('details-table')).toBeInTheDocument();
  });

  it('virtual list container has total height = entries * rowHeight', () => {
    const entries = generateEntries(500);

    renderWithHeight(
      <FileBrowser
        path="C:\\test"
        entries={entries}
        loading={false}
        error={null}
        isTauriRuntime={true}
        onOpenEntry={vi.fn()}
      />
    );

    const listContainer = screen.getByTestId('file-list');
    const spacer = listContainer.querySelector('div');
    expect(spacer).not.toBeNull();
    expect(spacer!.getAttribute('style')).toContain('height: 18000px');
    expect(spacer!.getAttribute('style')).toContain('position: relative');
  });

  it('details table has virtual spacer with correct total height', () => {
    const entries = generateEntries(500);

    renderWithHeight(
      <FileBrowser
        path="C:\\test"
        entries={entries}
        loading={false}
        error={null}
        isTauriRuntime={true}
        onOpenEntry={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /details view/i }));

    const container = screen.getByTestId('details-table');
    const spacer = container.children[1];
    expect(spacer).not.toBeNull();
    expect(spacer.getAttribute('style')).toContain('height: 22000px');
  });

  it('shows view mode buttons and switches between them', () => {
    const entries = generateEntries(10);

    renderWithHeight(
      <FileBrowser
        path="C:\\test"
        entries={entries}
        loading={false}
        error={null}
        isTauriRuntime={true}
        onOpenEntry={vi.fn()}
      />
    );

    expect(screen.getByRole('button', { name: /list view/i })).toHaveAttribute('aria-pressed', 'true');

    fireEvent.click(screen.getByRole('button', { name: /icon view/i }));
    expect(screen.getByRole('button', { name: /icon view/i })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByTestId('file-grid')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /details view/i }));
    expect(screen.getByRole('button', { name: /details view/i })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByTestId('details-table')).toBeInTheDocument();
  });
});
