import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from '../App';
import FileBrowser from '../components/files/FileBrowser';

describe('runtime fallback behavior', () => {
  it('shows a clear browser preview fallback notice', () => {
    render(<App />);

    expect(screen.getByText('Browser preview mode: desktop runtime features are limited.')).toBeInTheDocument();
    expect(screen.queryByText('No drives available')).not.toBeInTheDocument();
  });

  it('uses clean loading copy without encoding artifacts', () => {
    render(
      <FileBrowser
        path="C:\\Users"
        entries={[]}
        loading
        error={null}
        isTauriRuntime={false}
        onOpenEntry={() => {}}
      />,
    );

    expect(screen.getByText('Loading directory...')).toBeInTheDocument();
  });

  it('explains non-filesystem preview states clearly', () => {
    render(
      <FileBrowser
        path="This PC"
        entries={[]}
        loading={false}
        error={null}
        isTauriRuntime={false}
        onOpenEntry={() => {}}
      />,
    );

    expect(
      screen.getByText('This location is a preview entry. Browse real folders in the desktop app runtime.'),
    ).toBeInTheDocument();
  });
});
