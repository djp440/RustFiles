import { describe, expect, it } from 'vitest';
import {
  createTabState,
  getBreadcrumbSegments,
  goBackInTab,
  goForwardInTab,
  navigateTabToBreadcrumb,
  navigateTabToEntry,
  submitTabPath,
  type DirectoryEntry,
} from '../stores/tabs';

const folderEntry = (path: string, name: string): DirectoryEntry => ({
  path,
  name,
  size: 0,
  modified: 0,
  isHidden: false,
  isFolder: true,
});

describe('tab navigation state', () => {
  it('updates the current path when entering a directory', () => {
    const rootPath = 'C:\\Users\\demo';
    const nextPath = 'C:\\Users\\demo\\Documents';
    const tab = createTabState(rootPath);

    const updated = navigateTabToEntry(tab, folderEntry(nextPath, 'Documents'));

    expect(updated.path).toBe(nextPath);
    expect(updated.history).toEqual([rootPath, nextPath]);
    expect(updated.historyIndex).toBe(1);
  });

  it('moves backward and forward through history', () => {
    const firstPath = 'C:\\Users\\demo';
    const secondPath = 'C:\\Users\\demo\\Documents';
    const thirdPath = 'C:\\Users\\demo\\Documents\\Projects';

    const initial = createTabState(firstPath);
    const afterSecond = navigateTabToEntry(initial, folderEntry(secondPath, 'Documents'));
    const afterThird = navigateTabToEntry(afterSecond, folderEntry(thirdPath, 'Projects'));

    const afterBack = goBackInTab(afterThird);
    expect(afterBack.path).toBe(secondPath);
    expect(afterBack.historyIndex).toBe(1);

    const afterForward = goForwardInTab(afterBack);
    expect(afterForward.path).toBe(thirdPath);
    expect(afterForward.historyIndex).toBe(2);
  });

  it('updates path and history when submitting a path input', () => {
    const rootPath = 'C:\\Users\\demo';
    const submittedPath = 'C:\\Users\\demo\\Pictures';
    const tab = createTabState(rootPath);

    const updated = submitTabPath(tab, submittedPath);

    expect(updated.path).toBe(submittedPath);
    expect(updated.history).toEqual([rootPath, submittedPath]);
    expect(updated.historyIndex).toBe(1);
  });

  it('navigates to the clicked breadcrumb level', () => {
    const initial = createTabState('C:\\Users\\demo');
    const deepTab = submitTabPath(initial, 'C:\\Users\\demo\\Projects\\RustFiles');
    const segments = getBreadcrumbSegments(deepTab.path);

    const updated = navigateTabToBreadcrumb(deepTab, segments[2].path);

    expect(segments.map((segment) => segment.label)).toEqual([
      'C:',
      'Users',
      'demo',
      'Projects',
      'RustFiles',
    ]);
    expect(updated.path).toBe('C:\\Users\\demo');
    expect(updated.historyIndex).toBe(2);
    expect(updated.history).toEqual([
      'C:\\Users\\demo',
      'C:\\Users\\demo\\Projects\\RustFiles',
      'C:\\Users\\demo',
    ]);
  });
});
