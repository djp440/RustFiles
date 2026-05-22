import { invoke } from '@tauri-apps/api/core';

export interface VisibleRange {
  start_index: number;
  end_index: number;
}

export interface DirectoryEntry {
  path: string;
  name: string;
  size: number;
  modified: number;
  isHidden: boolean;
  isFolder: boolean;
}

export interface DirectoryPage {
  path: string;
  entries: DirectoryEntry[];
  totalCount: number;
  sortKey: 'name' | 'modified' | 'size' | 'type';
  sortAscending: boolean;
  filterKind: 'all' | 'folders' | 'files' | 'images' | 'documents' | 'videos';
  showHidden: boolean;
  snapshotVersion: number;
  offset: number;
  limit: number;
}

export type SchedulerReportKind = 'viewport' | 'interaction';

export type WorkKind =
  | 'foreground_directory_enumeration'
  | 'visible_thumbnail'
  | 'foreground_search'
  | 'file_progress'
  | 'background_refresh'
  | 'non_visible_thumbnail'
  | 'background_recursive_search';

export interface SchedulerSignal {
  active_tab_id: string;
  visible_range: VisibleRange | null;
  last_input_at_ms: number | null;
  is_scrolling: boolean;
  interaction_epoch: number;
}

export interface SchedulerReportAck {
  accepted: boolean;
  report_kind: SchedulerReportKind;
  active_tab_id: string;
  interaction_epoch: number;
  visible_range: VisibleRange | null;
  priority_order: WorkKind[];
  interaction_latency_ms: number | null;
  frame_budget_degraded: boolean;
  stale_interaction_epoch: boolean;
  tracked_interaction_epoch: number;
  summary: string;
}

export interface SchedulerDebugReport {
  report_kind: SchedulerReportKind;
  signal: SchedulerSignal;
  ack: SchedulerReportAck;
  recorded_at_ms: number;
}

export interface SchedulerDebugState {
  reports: SchedulerDebugReport[];
  viewport_reports: SchedulerDebugReport[];
  interaction_reports: SchedulerDebugReport[];
  latest_viewport_ack: SchedulerReportAck | null;
  latest_interaction_ack: SchedulerReportAck | null;
  latest_report: SchedulerDebugReport | null;
}

export interface ListDirectoryOptions {
  sortKey?: DirectoryPage['sortKey'];
  sortAscending?: boolean;
  filterKind?: DirectoryPage['filterKind'];
  showHidden?: boolean;
  offset?: number;
  limit?: number;
}

export interface SidebarRoots {
  desktop: string;
  downloads: string;
  documents: string;
  pictures: string;
  videos: string;
  music: string;
  thisPc: string;
}

interface RawSidebarRoots {
  desktop: string;
  downloads: string;
  documents: string;
  pictures: string;
  videos: string;
  music: string;
  this_pc?: string;
  thisPc?: string;
}

export interface DriveInfo {
  name: string;
  path: string;
  availableSpace: number;
  totalSpace: number;
}

export interface DriveList {
  drives: DriveInfo[];
}

export interface Settings {
  schemaVersion: number;
  showHiddenFiles: boolean;
  showFileExtensions: boolean;
  sortKey: 'name' | 'modified' | 'size' | 'type';
  sortAscending: boolean;
}

export type TaskStatus =
  | 'queued'
  | 'validating'
  | 'running'
  | 'waiting_for_conflict_decision'
  | 'cancelling'
  | 'cancelled'
  | 'completed'
  | 'failed'
  | 'partially_completed';

export interface AppError {
  code: string;
  message: string;
  retryable: boolean;
  refresh_suggestion: string | null;
}

export interface SearchRequest {
  root_path: string;
  query: string;
  recursive: boolean;
  snapshot_version: number | null;
}

export interface SearchErrorSummary {
  path: string;
  error: AppError;
}

export interface SearchResultBatch {
  task_id: string;
  root_path: string;
  query: string;
  recursive: boolean;
  snapshot_version: number | null;
  matches: DirectoryEntry[];
  error_summaries: SearchErrorSummary[];
}

export interface SearchTaskSnapshot {
  task_id: string;
  request: SearchRequest;
  status: TaskStatus;
  batches: SearchResultBatch[];
}

interface RawSettings {
  schema_version?: number;
  show_hidden_files?: boolean;
  show_file_extensions?: boolean;
  sort_key?: Settings['sortKey'];
  sort_ascending?: boolean;
}

interface SchedulerDebugTarget {
  __RUSTFILES_SCHEDULER_DEBUG__?: SchedulerDebugState;
}

interface SchedulerReportingTarget {
  __RUSTFILES_REPORT_VIEWPORT_STATE__?: typeof reportViewportState;
  __RUSTFILES_REPORT_INTERACTION_STATE__?: typeof reportInteractionState;
}

const FALLBACK_ROOTS: SidebarRoots = {
  desktop: 'Desktop',
  downloads: 'Downloads',
  documents: 'Documents',
  pictures: 'Pictures',
  videos: 'Videos',
  music: 'Music',
  thisPc: 'This PC',
};

const FALLBACK_DRIVES: DriveList = {
  drives: [],
};

const FALLBACK_SETTINGS: Settings = {
  schemaVersion: 1,
  showHiddenFiles: false,
  showFileExtensions: true,
  sortKey: 'name',
  sortAscending: true,
};

const PREVIEW_DIRECTORY_MAP: Record<string, Array<{ path: string; name: string; isFolder: boolean }>> = {
  'C:\\Users\\demo': [
    { path: 'C:\\Users\\demo\\Projects', name: 'Projects', isFolder: true },
    { path: 'C:\\Users\\demo\\Documents', name: 'Documents', isFolder: true },
    { path: 'C:\\Users\\demo\\Archive', name: 'Archive', isFolder: true },
    { path: 'C:\\Users\\demo\\report-root.txt', name: 'report-root.txt', isFolder: false },
    { path: 'C:\\Users\\demo\\notes.txt', name: 'notes.txt', isFolder: false },
  ],
  'C:\\Users\\demo\\Projects': [
    { path: 'C:\\Users\\demo\\Projects\\RustFiles', name: 'RustFiles', isFolder: true },
    { path: 'C:\\Users\\demo\\Projects\\Legacy', name: 'Legacy', isFolder: true },
    { path: 'C:\\Users\\demo\\Projects\\report-project.txt', name: 'report-project.txt', isFolder: false },
  ],
  'C:\\Users\\demo\\Projects\\RustFiles': [
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src', name: 'src', isFolder: true },
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\docs', name: 'docs', isFolder: true },
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\search-notes.txt', name: 'search-notes.txt', isFolder: false },
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\report-plan.md', name: 'report-plan.md', isFolder: false },
  ],
  'C:\\Users\\demo\\Projects\\RustFiles\\src': [
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\search.ts', name: 'search.ts', isFolder: false },
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\search-store.ts', name: 'search-store.ts', isFolder: false },
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep', name: 'deep', isFolder: true },
  ],
  'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep': [
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep\\nested-report.txt', name: 'nested-report.txt', isFolder: false },
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\src\\deep\\cancel-token.txt', name: 'cancel-token.txt', isFolder: false },
  ],
  'C:\\Users\\demo\\Projects\\RustFiles\\docs': [
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\docs\\search-spec.md', name: 'search-spec.md', isFolder: false },
    { path: 'C:\\Users\\demo\\Projects\\RustFiles\\docs\\recursion-notes.md', name: 'recursion-notes.md', isFolder: false },
  ],
  'C:\\Users\\demo\\Documents': [
    { path: 'C:\\Users\\demo\\Documents\\report-draft.txt', name: 'report-draft.txt', isFolder: false },
    { path: 'C:\\Users\\demo\\Documents\\todo.txt', name: 'todo.txt', isFolder: false },
  ],
  'C:\\Users\\demo\\Archive': [
    { path: 'C:\\Users\\demo\\Archive\\archived-report.txt', name: 'archived-report.txt', isFolder: false },
  ],
  'C:\\Users\\demo\\Projects\\Legacy': [
    { path: 'C:\\Users\\demo\\Projects\\Legacy\\legacy-report.txt', name: 'legacy-report.txt', isFolder: false },
  ],
};

let previewSnapshotVersion = 0;

const SCHEDULER_PRIORITY_ORDER: WorkKind[] = [
  'foreground_directory_enumeration',
  'visible_thumbnail',
  'foreground_search',
  'file_progress',
  'background_refresh',
  'non_visible_thumbnail',
  'background_recursive_search',
];

const tabInteractionEpochs = new Map<string, number>();

function getSchedulerDebugTarget(): SchedulerDebugTarget {
  return globalThis as unknown as SchedulerDebugTarget;
}

function getSchedulerReportingTarget(): SchedulerReportingTarget {
  return globalThis as unknown as SchedulerReportingTarget;
}

function createEmptySchedulerDebugState(): SchedulerDebugState {
  return {
    reports: [],
    viewport_reports: [],
    interaction_reports: [],
    latest_viewport_ack: null,
    latest_interaction_ack: null,
    latest_report: null,
  };
}

function getSchedulerDebugState(): SchedulerDebugState {
  const target = getSchedulerDebugTarget();
  if (!target.__RUSTFILES_SCHEDULER_DEBUG__) {
    target.__RUSTFILES_SCHEDULER_DEBUG__ = createEmptySchedulerDebugState();
  }

  return target.__RUSTFILES_SCHEDULER_DEBUG__;
}

export function resetSchedulerDebugState() {
  tabInteractionEpochs.clear();
  const target = getSchedulerDebugTarget();
  target.__RUSTFILES_SCHEDULER_DEBUG__ = createEmptySchedulerDebugState();
}

function updateSchedulerDebugState(report: SchedulerDebugReport) {
  const state = getSchedulerDebugState();
  state.reports.push(report);
  state.latest_report = report;

  if (report.report_kind === 'viewport') {
    state.viewport_reports.push(report);
    state.latest_viewport_ack = report.ack;
  } else {
    state.interaction_reports.push(report);
    state.latest_interaction_ack = report.ack;
  }
}

function visibleSpan(visibleRange: VisibleRange | null): number | null {
  if (!visibleRange) {
    return null;
  }

  return visibleRange.end_index - visibleRange.start_index + 1;
}

function priorityScore(signal: SchedulerSignal, workKind: WorkKind): number {
  const span = visibleSpan(signal.visible_range);

  switch (workKind) {
    case 'foreground_directory_enumeration':
      return 0;
    case 'visible_thumbnail':
      return span && span > 0 ? 10 : 12;
    case 'foreground_search':
      return 20;
    case 'file_progress':
      return 30;
    case 'background_refresh':
      return signal.is_scrolling ? 50 : 40;
    case 'non_visible_thumbnail':
      return signal.is_scrolling ? 110 : 50;
    case 'background_recursive_search':
      return signal.is_scrolling ? 130 : 60;
    default:
      return 999;
  }
}

function priorityOrder(signal: SchedulerSignal): WorkKind[] {
  return [...SCHEDULER_PRIORITY_ORDER].sort((left, right) => {
    const scoreDelta = priorityScore(signal, left) - priorityScore(signal, right);
    if (scoreDelta !== 0) {
      return scoreDelta;
    }

    return SCHEDULER_PRIORITY_ORDER.indexOf(left) - SCHEDULER_PRIORITY_ORDER.indexOf(right);
  });
}

function summarizeAck(
  signal: SchedulerSignal,
  reportKind: SchedulerReportKind,
  interactionLatencyMs: number | null,
  frameBudgetDegraded: boolean,
): SchedulerReportAck {
  const order = priorityOrder(signal);
  const trackedInteractionEpoch = tabInteractionEpochs.get(signal.active_tab_id) ?? 0;
  const staleInteractionEpoch = signal.interaction_epoch < trackedInteractionEpoch;

  if (!staleInteractionEpoch) {
    tabInteractionEpochs.set(signal.active_tab_id, signal.interaction_epoch);
  }

  const summary =
    reportKind === 'viewport'
      ? `viewport ack for tab '${signal.active_tab_id}' with ${order.length} priority tiers${
          frameBudgetDegraded ? ', frame budget degraded' : ''
        }${staleInteractionEpoch ? ', stale interaction epoch detected' : ''}`
      : `interaction ack for tab '${signal.active_tab_id}' with latency ${
          interactionLatencyMs === null ? 'unknown' : `${interactionLatencyMs}ms`
        }${frameBudgetDegraded ? ', frame budget degraded' : ''}`;

  const ack: SchedulerReportAck = {
    accepted: true,
    report_kind: reportKind,
    active_tab_id: signal.active_tab_id,
    interaction_epoch: signal.interaction_epoch,
    visible_range: signal.visible_range,
    priority_order: order,
    interaction_latency_ms: interactionLatencyMs,
    frame_budget_degraded: frameBudgetDegraded,
    stale_interaction_epoch: staleInteractionEpoch,
    tracked_interaction_epoch: tabInteractionEpochs.get(signal.active_tab_id) ?? 0,
    summary,
  };

  updateSchedulerDebugState({
    report_kind: reportKind,
    signal,
    ack,
    recorded_at_ms: Date.now(),
  });

  return ack;
}

function reportSignal(
  reportKind: SchedulerReportKind,
  signal: SchedulerSignal,
): SchedulerReportAck {
  const nowMs = Date.now();
  const interactionLatencyMs =
    signal.last_input_at_ms === null ? null : Math.max(0, nowMs - signal.last_input_at_ms);
  const span = visibleSpan(signal.visible_range);

  const frameBudgetDegraded =
    reportKind === 'viewport' ? signal.is_scrolling || (span !== null && span >= 128) : signal.is_scrolling;

  return summarizeAck(signal, reportKind, interactionLatencyMs, frameBudgetDegraded);
}

async function invokeSchedulerReport(
  command: 'report_viewport_state' | 'report_interaction_state',
  signal: SchedulerSignal,
): Promise<SchedulerReportAck> {
  if (!hasTauriRuntime()) {
    return reportSignal(command === 'report_viewport_state' ? 'viewport' : 'interaction', signal);
  }

  const ack = await invoke<SchedulerReportAck>(command, {
    signal,
  });

  return ack;
}

function toCamelDirectoryPage(page: {
  path: string;
  entries: DirectoryEntry[];
  total_count?: number;
  totalCount?: number;
  sort_key?: DirectoryPage['sortKey'];
  sortKey?: DirectoryPage['sortKey'];
  sort_ascending?: boolean;
  sortAscending?: boolean;
  filter_kind?: DirectoryPage['filterKind'];
  filterKind?: DirectoryPage['filterKind'];
  show_hidden?: boolean;
  showHidden?: boolean;
  snapshot_version?: number;
  snapshotVersion?: number;
  offset?: number;
  limit?: number;
}): DirectoryPage {
  return {
    path: page.path,
    entries: page.entries,
    totalCount: page.totalCount ?? page.total_count ?? page.entries.length,
    sortKey: page.sortKey ?? page.sort_key ?? 'name',
    sortAscending: page.sortAscending ?? page.sort_ascending ?? true,
    filterKind: page.filterKind ?? page.filter_kind ?? 'all',
    showHidden: page.showHidden ?? page.show_hidden ?? false,
    snapshotVersion: page.snapshotVersion ?? page.snapshot_version ?? 0,
    offset: page.offset ?? 0,
    limit: page.limit ?? Number.MAX_SAFE_INTEGER,
  };
}

function toCamelSidebarRoots(roots: RawSidebarRoots): SidebarRoots {
  return {
    desktop: roots.desktop,
    downloads: roots.downloads,
    documents: roots.documents,
    pictures: roots.pictures,
    videos: roots.videos,
    music: roots.music,
    thisPc: roots.thisPc ?? roots.this_pc ?? 'This PC',
  };
}

function toCamelSettings(settings: RawSettings): Settings {
  return {
    schemaVersion: settings.schema_version ?? 1,
    showHiddenFiles: settings.show_hidden_files ?? false,
    showFileExtensions: settings.show_file_extensions ?? true,
    sortKey: settings.sort_key ?? 'name',
    sortAscending: settings.sort_ascending ?? true,
  };
}

function toRawSettings(settings: Settings): RawSettings {
  return {
    schema_version: settings.schemaVersion,
    show_hidden_files: settings.showHiddenFiles,
    show_file_extensions: settings.showFileExtensions,
    sort_key: settings.sortKey,
    sort_ascending: settings.sortAscending,
  };
}

export function hasTauriRuntime(): boolean {
  return typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
}

export async function reportViewportState(signal: SchedulerSignal): Promise<SchedulerReportAck> {
  return invokeSchedulerReport('report_viewport_state', signal);
}

export async function reportInteractionState(signal: SchedulerSignal): Promise<SchedulerReportAck> {
  return invokeSchedulerReport('report_interaction_state', signal);
}

const schedulerReportingTarget = getSchedulerReportingTarget();
schedulerReportingTarget.__RUSTFILES_REPORT_VIEWPORT_STATE__ = reportViewportState;
schedulerReportingTarget.__RUSTFILES_REPORT_INTERACTION_STATE__ = reportInteractionState;

export async function listDirectory(path: string, options: ListDirectoryOptions = {}): Promise<DirectoryPage> {
  if (!hasTauriRuntime()) {
    const normalizedPath = path.trim();
    if (!isFilesystemPath(normalizedPath)) {
      return toCamelDirectoryPage({
        path,
        entries: [],
        sortKey: options.sortKey,
        sortAscending: options.sortAscending,
        filterKind: options.filterKind,
        showHidden: options.showHidden,
        offset: options.offset,
        limit: options.limit,
        snapshotVersion: 0,
      });
    }

    previewSnapshotVersion += 1;

    const rawEntries =
      PREVIEW_DIRECTORY_MAP[normalizedPath] ??
      buildPreviewDirectoryEntries(normalizedPath);

    return toCamelDirectoryPage({
      path,
      entries: rawEntries.map((entry, index) => ({
        path: entry.path,
        name: entry.name,
        size: entry.isFolder ? 0 : 1024 + index,
        modified: 1_700_000_000 + index,
        isHidden: false,
        isFolder: entry.isFolder,
      })),
      sortKey: options.sortKey,
      sortAscending: options.sortAscending,
      filterKind: options.filterKind,
      showHidden: options.showHidden,
      offset: options.offset,
      limit: options.limit,
      snapshotVersion: previewSnapshotVersion,
    });
  }

  try {
    const page = await invoke<{
      path: string;
      entries: DirectoryEntry[];
      total_count: number;
      sort_key: DirectoryPage['sortKey'];
      sort_ascending: boolean;
      filter_kind: DirectoryPage['filterKind'];
      show_hidden: boolean;
      snapshot_version: number;
      offset: number;
      limit: number;
    }>('list_directory', {
      path,
      sortKey: options.sortKey,
      sortAscending: options.sortAscending,
      filterKind: options.filterKind,
      showHidden: options.showHidden,
      offset: options.offset,
      limit: options.limit,
    });

    return toCamelDirectoryPage(page);
  } catch {
    return toCamelDirectoryPage({
      path,
      entries: [],
    });
  }
}

export async function startSearch(request: SearchRequest): Promise<string> {
  if (!hasTauriRuntime()) {
    return `preview-search-${Date.now()}`;
  }

  return await invoke<string>('start_search', {
    request,
  });
}

export async function cancelTask(taskId: string): Promise<TaskStatus> {
  if (!hasTauriRuntime()) {
    return 'cancelled';
  }

  return await invoke<TaskStatus>('cancel_task', {
    taskId,
  });
}

function isFilesystemPath(path: string): boolean {
  return /^[A-Za-z]:\\/.test(path) || path.startsWith('\\\\');
}

function buildPreviewDirectoryEntries(path: string): Array<{ path: string; name: string; isFolder: boolean }> {
  const pathParts = path.split('\\').filter(Boolean);
  const leaf = pathParts.length > 0 ? pathParts[pathParts.length - 1] : 'Preview';
  const folderStem = leaf === ':' ? 'Drive' : leaf.replace(/[:]/g, '');

  return [
    { path: `${path}\\${folderStem}-alpha`, name: `${folderStem}-alpha`, isFolder: true },
    { path: `${path}\\${folderStem}-beta`, name: `${folderStem}-beta`, isFolder: true },
    { path: `${path}\\report-${folderStem}.txt`, name: `report-${folderStem}.txt`, isFolder: false },
    { path: `${path}\\search-${folderStem}.txt`, name: `search-${folderStem}.txt`, isFolder: false },
    { path: `${path}\\notes-${folderStem}.txt`, name: `notes-${folderStem}.txt`, isFolder: false },
  ];
}

export async function getSidebarRoots(): Promise<SidebarRoots> {
  if (!hasTauriRuntime()) {
    return FALLBACK_ROOTS;
  }

  try {
    const roots = await invoke<RawSidebarRoots>('get_sidebar_roots');

    return toCamelSidebarRoots(roots);
  } catch {
    return FALLBACK_ROOTS;
  }
}

export async function getDrives(): Promise<DriveList> {
  if (!hasTauriRuntime()) {
    return FALLBACK_DRIVES;
  }

  try {
    return await invoke<DriveList>('get_drives');
  } catch {
    return FALLBACK_DRIVES;
  }
}

export async function getSettings(): Promise<Settings> {
  if (!hasTauriRuntime()) {
    return FALLBACK_SETTINGS;
  }

  try {
    const settings = await invoke<RawSettings>('get_settings');

    return toCamelSettings(settings);
  } catch {
    return FALLBACK_SETTINGS;
  }
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  if (!hasTauriRuntime()) {
    return settings;
  }

  try {
    const updatedSettings = await invoke<RawSettings>('update_settings', {
      settings: toRawSettings(settings),
    });

    return toCamelSettings(updatedSettings);
  } catch {
    return settings;
  }
}
