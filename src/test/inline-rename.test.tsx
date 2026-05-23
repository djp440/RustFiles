import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import InlineRename from '../components/files/InlineRename';

const RESERVED_NAMES = [
  'CON',
  'PRN',
  'AUX',
  'NUL',
  'COM1',
  'COM2',
  'COM3',
  'COM4',
  'COM5',
  'COM6',
  'COM7',
  'COM8',
  'COM9',
  'LPT1',
  'LPT2',
  'LPT3',
  'LPT4',
  'LPT5',
  'LPT6',
  'LPT7',
  'LPT8',
  'LPT9',
];

const ILLEGAL_CHARS = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

describe('InlineRename', () => {
  it('renders an input pre-filled with initialName and auto-focuses', () => {
    render(
      <InlineRename
        initialName="report.txt"
        existingNames={[]}
        isFolder={false}
        onCommit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole('textbox');
    expect(input).toHaveFocus();
    expect(input).toHaveValue('report.txt');
  });

  it('commits new name on Enter when name is valid', () => {
    const onCommit = vi.fn();
    const onCancel = vi.fn();

    render(
      <InlineRename
        initialName="old.txt"
        existingNames={['other.txt']}
        isFolder={false}
        onCommit={onCommit}
        onCancel={onCancel}
      />,
    );

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'renamed.txt' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onCommit).toHaveBeenCalledWith('renamed.txt');
    expect(onCancel).not.toHaveBeenCalled();
  });

  it('cancels on Escape without calling onCommit', () => {
    const onCommit = vi.fn();
    const onCancel = vi.fn();

    render(
      <InlineRename
        initialName="test.txt"
        existingNames={[]}
        isFolder={false}
        onCommit={onCommit}
        onCancel={onCancel}
      />,
    );

    const input = screen.getByRole('textbox');
    fireEvent.keyDown(input, { key: 'Escape' });

    expect(onCommit).not.toHaveBeenCalled();
    expect(onCancel).toHaveBeenCalled();
  });

  it('does not commit when name is empty', () => {
    const onCommit = vi.fn();

    render(
      <InlineRename
        initialName="test.txt"
        existingNames={[]}
        isFolder={false}
        onCommit={onCommit}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onCommit).not.toHaveBeenCalled();
  });

  it('rejects illegal characters and shows error', () => {
    for (const char of ILLEGAL_CHARS) {
      const onCommit = vi.fn();
      const { unmount } = render(
        <InlineRename
          initialName="test.txt"
          existingNames={[]}
          isFolder={false}
          onCommit={onCommit}
          onCancel={vi.fn()}
        />,
      );

      const input = screen.getByRole('textbox');
      fireEvent.change(input, { target: { value: `bad${char}name` } });
      fireEvent.keyDown(input, { key: 'Enter' });

      expect(onCommit).not.toHaveBeenCalled();
      expect(screen.getByText(/非法字符/)).toBeInTheDocument();

      unmount();
    }
  });

  it('rejects reserved Windows names and shows error', () => {
    for (const reserved of RESERVED_NAMES) {
      const onCommit = vi.fn();
      const { unmount } = render(
        <InlineRename
          initialName="test.txt"
          existingNames={[]}
          isFolder={false}
          onCommit={onCommit}
          onCancel={vi.fn()}
        />,
      );

      const input = screen.getByRole('textbox');
      fireEvent.change(input, { target: { value: reserved } });
      fireEvent.keyDown(input, { key: 'Enter' });

      expect(onCommit).not.toHaveBeenCalled();
      expect(screen.getByText(/保留名/)).toBeInTheDocument();

      unmount();
    }
  });

  it('rejects reserved names case-insensitively', () => {
    const onCommit = vi.fn();

    render(
      <InlineRename
        initialName="test.txt"
        existingNames={[]}
        isFolder={false}
        onCommit={onCommit}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'con' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onCommit).not.toHaveBeenCalled();
    expect(screen.getByText(/保留名/)).toBeInTheDocument();
  });

  it('rejects reserved names with extension (e.g. CON.txt)', () => {
    const onCommit = vi.fn();

    render(
      <InlineRename
        initialName="test.txt"
        existingNames={[]}
        isFolder={false}
        onCommit={onCommit}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'CON.txt' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onCommit).not.toHaveBeenCalled();
    expect(screen.getByText(/保留名/)).toBeInTheDocument();
  });

  it('rejects duplicate name and shows error', () => {
    const onCommit = vi.fn();

    render(
      <InlineRename
        initialName="test.txt"
        existingNames={['already.txt', 'report.txt']}
        isFolder={false}
        onCommit={onCommit}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'already.txt' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onCommit).not.toHaveBeenCalled();
    expect(screen.getByText(/同名/)).toBeInTheDocument();
  });

  it('allows renaming to the same name as original', () => {
    const onCommit = vi.fn();

    render(
      <InlineRename
        initialName="test.txt"
        existingNames={['test.txt', 'other.txt']}
        isFolder={false}
        onCommit={onCommit}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'test.txt' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onCommit).toHaveBeenCalledWith('test.txt');
  });

  it('clears error when user changes input after an error', () => {
    const onCommit = vi.fn();

    render(
      <InlineRename
        initialName="test.txt"
        existingNames={[]}
        isFolder={false}
        onCommit={onCommit}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole('textbox');

    fireEvent.change(input, { target: { value: 'bad<name.txt' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(screen.getByText(/非法字符/)).toBeInTheDocument();

    fireEvent.change(input, { target: { value: 'good-name.txt' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onCommit).toHaveBeenCalledWith('good-name.txt');
  });
});
