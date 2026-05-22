import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import FileBrowser from '../components/files/FileBrowser';

const MOCK_ENTRIES = [
  {
    path: 'C:\\test\\folder',
    name: 'folder',
    size: 0,
    modified: 1000,
    isHidden: false,
    isFolder: true,
  },
  {
    path: 'C:\\test\\file.txt',
    name: 'file.txt',
    size: 1024,
    modified: 2000,
    isHidden: false,
    isFolder: false,
  },
];

describe('FileBrowser View Modes', () => {
  it('defaults to list view', () => {
    render(
      <FileBrowser
        path="C:\\test"
        entries={MOCK_ENTRIES}
        loading={false}
        error={null}
        isTauriRuntime={true}
        onOpenEntry={vi.fn()}
      />
    );

    expect(screen.getByRole('button', { name: /list view/i })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByTestId('file-list')).toBeInTheDocument();
  });

  it('switches between views and maintains selection', () => {
    render(
      <FileBrowser
        path="C:\\test"
        entries={MOCK_ENTRIES}
        loading={false}
        error={null}
        isTauriRuntime={true}
        onOpenEntry={vi.fn()}
      />
    );

    // Select an item in list view
    const folderItem = screen.getByText('folder');
    fireEvent.click(folderItem);

    // Switch to icon view
    fireEvent.click(screen.getByRole('button', { name: /icon view/i }));
    expect(screen.getByRole('button', { name: /icon view/i })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByTestId('file-grid')).toBeInTheDocument();
    
    // Selection should be preserved
    expect(screen.getByLabelText(/folder/i)).toHaveAttribute('aria-selected', 'true');

    // Switch to details view
    fireEvent.click(screen.getByRole('button', { name: /details view/i }));
    expect(screen.getByRole('button', { name: /details view/i })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByTestId('details-table')).toBeInTheDocument();

    // Selection should still be preserved
    expect(screen.getByLabelText(/folder/i)).toHaveAttribute('aria-selected', 'true');
  });

  it('renders GlassSurface with correct variant', () => {
    render(
      <FileBrowser
        path="C:\\test"
        entries={MOCK_ENTRIES}
        loading={false}
        error={null}
        isTauriRuntime={true}
        onOpenEntry={vi.fn()}
      />
    );

    const surface = screen.getByRole('region', { name: /file browser/i });
    expect(surface).toHaveAttribute('data-surface-variant', 'content');
  });
});
