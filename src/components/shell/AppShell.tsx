import { useEffect, useState } from 'react';
import {
  getDrives,
  getSidebarRoots,
  hasTauriRuntime,
  listDirectory,
  type DirectoryPage,
  type DriveInfo,
  type SidebarRoots,
} from '../../api/tauri';
import FileBrowser from '../files/FileBrowser';
import TaskPanel from '../tasks/TaskPanel';
import NavigationBar from '../navigation/NavigationBar';
import Toolbar from '../toolbar/Toolbar';
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
import { searchStore } from '../../stores/search';
import { loadSettings, saveSettings, settingsStore } from '../../stores/settings';
import { useInteractionReporting } from '../../hooks/useInteractionReporting';

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
  const isTauriRuntime = hasTauriRuntime();
  const [tab, setTab] = useState(() => createTabState('This PC'));
  const [roots, setRoots] = useState<SidebarRoots>(INITIAL_ROOTS);
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [settingsState, setSettingsState] = useState(() => settingsStore.getState());
  const [filterKind, setFilterKind] = useState<DirectoryPage['filterKind']>('all');
  const [settingsReady, setSettingsReady] = useState(false);
  const [currentSnapshotVersion, setCurrentSnapshotVersion] = useState<number | null>(null);
  const { interactionEpoch, lastInputAtMs, reportInteraction } = useInteractionReporting({
    activeTabId: tab.id,
  });

  useEffect(() => {
    if (!isTauriRuntime) {
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
  }, [isTauriRuntime]);

  useEffect(() => {
    const unsubscribe = settingsStore.subscribe((nextState) => {
      setSettingsState(nextState);
    });

    setSettingsState(settingsStore.getState());

    let cancelled = false;

    void loadSettings().finally(() => {
      if (!cancelled) {
        setSettingsState(settingsStore.getState());
        setSettingsReady(true);
      }
    });

    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, []);

  async function loadDirectory(path: string) {
    if (!settingsReady) {
      return;
    }

    setCurrentSnapshotVersion(null);

    if (!isFilesystemPath(path)) {
      setTab((currentTab) => ({
        ...currentTab,
        entries: [],
        loading: false,
        error: null,
      }));
      return;
    }

    setTab((currentTab) => setTabLoading(currentTab, true));

    try {
      const page = await listDirectory(path, {
        sortKey: settingsState.settings.sortKey,
        sortAscending: settingsState.settings.sortAscending,
        filterKind,
        showHidden: settingsState.settings.showHiddenFiles,
      });
      setCurrentSnapshotVersion(page.snapshotVersion);
      setTab((currentTab) => applyDirectoryPage(currentTab, page));
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : 'Failed to load directory.';
      setTab((currentTab) => setTabError(currentTab, message));
    }
  }

  useEffect(() => {
    if (!settingsReady) {
      return undefined;
    }

    void loadDirectory(tab.path);
    return undefined;
  }, [filterKind, settingsReady, settingsState.settings.showHiddenFiles, settingsState.settings.sortAscending, settingsState.settings.sortKey, tab.path]);

  useEffect(() => {
    searchStore.setContext({
      currentPath: tab.path,
      currentSnapshotVersion,
      currentEntries: tab.entries,
      isTauriRuntime,
    });
  }, [currentSnapshotVersion, isTauriRuntime, tab.entries, tab.path]);

  function handleSortKeyChange(sortKey: DirectoryPage['sortKey']) {
    void saveSettings({
      ...settingsState.settings,
      sortKey,
    });
  }

  function handleSortDirectionToggle() {
    void saveSettings({
      ...settingsState.settings,
      sortAscending: !settingsState.settings.sortAscending,
    });
  }

  function handleShowHiddenFilesChange(showHiddenFiles: boolean) {
    void saveSettings({
      ...settingsState.settings,
      showHiddenFiles,
    });
  }

  function handleShowFileExtensionsChange(showFileExtensions: boolean) {
    void saveSettings({
      ...settingsState.settings,
      showFileExtensions,
    });
  }

  function handleSidebarSelectPath(path: string) {
    void reportInteraction();
    setTab((currentTab) => submitTabPath(currentTab, path));
  }

  function handleBack() {
    void reportInteraction();
    setTab((currentTab) => goBackInTab(currentTab));
  }

  function handleForward() {
    void reportInteraction();
    setTab((currentTab) => goForwardInTab(currentTab));
  }

  function handleSubmitPath(path: string) {
    void reportInteraction();
    setTab((currentTab) => submitTabPath(currentTab, path));
  }

  function handleBreadcrumbSelect(path: string) {
    void reportInteraction();
    setTab((currentTab) => navigateTabToBreadcrumb(currentTab, path));
  }

  function handleOpenEntry(entry: Parameters<typeof navigateTabToEntry>[1]) {
    setTab((currentTab) => navigateTabToEntry(currentTab, entry));
  }

  async function handleOpenSearchResultLocation(item: Parameters<typeof searchStore.openSearchResultLocation>[0]) {
    await searchStore.openSearchResultLocation(item, {
      currentPath: tab.path,
      currentSnapshotVersion,
      navigateToPath: async (nextPath) => {
        void reportInteraction();
        setTab((currentTab) => submitTabPath(currentTab, nextPath));
      },
      refreshCurrentDirectory: async () => {
        await loadDirectory(tab.path);
      },
    });
  }

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'grid',
        gridTemplateRows: 'auto auto auto auto 1fr',
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
        onBack={handleBack}
        onForward={handleForward}
        onSubmitPath={handleSubmitPath}
        onSelectBreadcrumb={handleBreadcrumbSelect}
      />
      <Toolbar
        sortKey={settingsState.settings.sortKey}
        sortAscending={settingsState.settings.sortAscending}
        filterKind={filterKind}
        showHiddenFiles={settingsState.settings.showHiddenFiles}
        showFileExtensions={settingsState.settings.showFileExtensions}
        onSortKeyChange={handleSortKeyChange}
        onSortDirectionToggle={handleSortDirectionToggle}
        onFilterKindChange={setFilterKind}
        onShowHiddenFilesChange={handleShowHiddenFilesChange}
        onShowFileExtensionsChange={handleShowFileExtensionsChange}
      />
      <TaskPanel />
      <div style={{ display: 'grid', gridTemplateColumns: '220px 1fr', minHeight: 0 }}>
        <Sidebar
          roots={roots}
          drives={drives}
          isTauriRuntime={isTauriRuntime}
          activePath={tab.path}
          onSelectPath={handleSidebarSelectPath}
        />
        <FileBrowser
          path={tab.path}
          activeTabId={tab.id}
          interactionEpoch={interactionEpoch}
          lastInputAtMs={lastInputAtMs}
          entries={tab.entries}
          loading={tab.loading}
          error={tab.error}
          isTauriRuntime={isTauriRuntime}
          sortKey={settingsState.settings.sortKey}
          sortAscending={settingsState.settings.sortAscending}
          filterKind={filterKind}
          showHiddenFiles={settingsState.settings.showHiddenFiles}
          showFileExtensions={settingsState.settings.showFileExtensions}
          onOpenEntry={handleOpenEntry}
          onOpenSearchResultLocation={handleOpenSearchResultLocation}
          onUserInteraction={() => void reportInteraction()}
        />
      </div>
    </div>
  );
}

export default AppShell;
