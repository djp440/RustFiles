type Listener = () => void;

export interface ClipboardOperation {
  operationId: string;
  type: 'copy' | 'cut';
  paths: string[];
}

interface SelectionState {
  selectedPaths: string[];
  clipboardOp: ClipboardOperation | null;
}

const initialState: SelectionState = {
  selectedPaths: [],
  clipboardOp: null,
};

let state: SelectionState = initialState;
const listeners = new Set<Listener>();

function emitChange() {
  for (const listener of listeners) {
    listener();
  }
}

function setState(next: Partial<SelectionState>) {
  state = {
    ...state,
    ...next,
  };
  emitChange();
}

let opCounter = 0;

function nextOperationId(): string {
  opCounter += 1;
  return `clip-${Date.now()}-${opCounter}`;
}

export const selectionStore: SelectionState & {
  subscribe: (listener: Listener) => () => boolean;
  getState: () => SelectionState;
  setSelectedPaths: (paths: string[]) => void;
  clearSelection: () => void;
  setClipboardCopy: (paths: string[]) => void;
  setClipboardCut: (paths: string[]) => void;
  clearClipboard: () => void;
  isCutPending: (path: string) => boolean;
} = {
  get selectedPaths() {
    return state.selectedPaths;
  },
  get clipboardOp() {
    return state.clipboardOp;
  },
  subscribe(listener: Listener) {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },
  getState: () => state,
  setSelectedPaths(paths: string[]) {
    setState({ selectedPaths: paths });
  },
  clearSelection() {
    setState({ selectedPaths: [] });
  },
  setClipboardCopy(paths: string[]) {
    setState({
      clipboardOp: {
        operationId: nextOperationId(),
        type: 'copy',
        paths: [...paths],
      },
    });
  },
  setClipboardCut(paths: string[]) {
    setState({
      clipboardOp: {
        operationId: nextOperationId(),
        type: 'cut',
        paths: [...paths],
      },
    });
  },
  clearClipboard() {
    setState({ clipboardOp: null, selectedPaths: [] });
  },
  isCutPending(path: string): boolean {
    if (!state.clipboardOp || state.clipboardOp.type !== 'cut') {
      return false;
    }
    return state.clipboardOp.paths.includes(path);
  },
};

export function resetSelectionStore() {
  state = { ...initialState };
  opCounter = 0;
  emitChange();
}
