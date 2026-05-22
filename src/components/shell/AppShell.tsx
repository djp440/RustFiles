import { useEffect, useState } from 'react';
import {
  getDrives,
  getSidebarRoots,
  hasTauriRuntime,
  listDirectory,
  type DriveInfo,
  type SidebarRoots,
} from '../../api/tauri';
import FileBrowser from '../files/FileBrowser';
import NavigationBar from '../navigation/NavigationBar';
import Sidebar from '../sidebar/Sidebar';
import WindowChrome from '../window/WindowChrome';
import {
  applyDirectoryPage,
  canGoBack,
  canGoForward,
  createTabState,
  getBreadcrumbSegments,
  goBackInTab,
  goForwardInTab,
  navigateTabToBreadcrumb,
  navigateTabToEntry,
  setTabError,
  setTabLoading,
  submitTabPath,
} from '../../stores/tabs';

const INITIAL_ROOTS: SidebarRoots = {
  desktop: 'Desktop',
  downloads: 'Downloads',
  documents: 'Documents',
  pictures: 'Pictures',
  videos: 'Videos',
  music: 'Music',
  thisPc: 'This PC',
};

function isFilesystemPath(path: string): boolean {
  return /^[A-Za-z]:\\/.test(path) || path.startsWith('\\\\');
}

function AppShell() {
  const [tab, setTab] = useState(() => createTabState('This PC'));
  const [roots, setRoots] = useState<SidebarRoots>(INITIAL_ROOTS);
  const [drives, setDrives] = useState<DriveInfo[]>([]);

  useEffect(() => {
    if (!hasTauriRuntime()) {
      return;
    }

    let cancelled = false;

    void getSidebarRoots().then((nextRoots) => {
      if (!cancelled) {
        setRoots(nextRoots);
      }
    });

    void getDrives().then((driveList) => {
      if (!cancelled) {
        setDrives(driveList.drives);
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!isFilesystemPath(tab.path)) {
      setTab((currentTab) => ({
        ...currentTab,
        entries: [],
        loading: false,
        error: null,
      }));
      return;
    }

    let cancelled = false;
    setTab((currentTab) => setTabLoading(currentTab, true));

    void listDirectory(tab.path)
      .then((page) => {
        if (!cancelled) {
          setTab((currentTab) => applyDirectoryPage(currentTab, page));
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : 'Failed to load directory.';
          setTab((currentTab) => setTabError(currentTab, message));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [tab.path]);

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'grid',
        gridTemplateRows: 'auto auto 1fr',
        background: '#111827',
        color: '#f9fafb',
        fontFamily: 'Segoe UI, sans-serif',
      }}
    >
      <WindowChrome />
      <NavigationBar
        path={tab.path}
        canGoBack={canGoBack(tab)}
        canGoForward={canGoForward(tab)}
        breadcrumbSegments={getBreadcrumbSegments(tab.path)}
        onBack={() => setTab((currentTab) => goBackInTab(currentTab))}
        onForward={() => setTab((currentTab) => goForwardInTab(currentTab))}
        onSubmitPath={(path) => setTab((currentTab) => submitTabPath(currentTab, path))}
        onSelectBreadcrumb={(path) =>
          setTab((currentTab) => navigateTabToBreadcrumb(currentTab, path))
        }
      />
      <div style={{ display: 'grid', gridTemplateColumns: '220px 1fr', minHeight: 0 }}>
        <Sidebar
          roots={roots}
          drives={drives}
          activePath={tab.path}
          onSelectPath={(path) => setTab((currentTab) => submitTabPath(currentTab, path))}
        />
        <FileBrowser
          path={tab.path}
          entries={tab.entries}
          loading={tab.loading}
          error={tab.error}
          onOpenEntry={(entry) => setTab((currentTab) => navigateTabToEntry(currentTab, entry))}
        />
      </div>
    </div>
  );
}

export default AppShell;
