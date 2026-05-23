import { useSyncExternalStore } from 'react';
import type { TaskSummary } from '../api/tauri';

interface TaskStoreState {
  tasks: TaskSummary[];
  expandedTaskIds: string[];
}

interface TaskStoreActions {
  setTasks: (tasks: TaskSummary[]) => void;
  toggleTaskExpanded: (taskId: string) => void;
  setTaskExpanded: (taskId: string, expanded: boolean) => void;
  setAllTasksExpanded: (expanded: boolean) => void;
}

type Listener = () => void;

const initialState: TaskStoreState = {
  tasks: [
    {
      id: 'task-preview-1',
      kind: 'copy_items',
      status: 'running',
      message: 'Copying 2 items',
      completed_items: ['a.txt'],
      incomplete_items: ['b.txt'],
      unknown_items: [],
      can_cancel: true,
    },
    {
      id: 'task-preview-2',
      kind: 'move_items',
      status: 'waiting_for_conflict_decision',
      message: 'Waiting for conflict decision',
      completed_items: ['done.txt'],
      incomplete_items: ['todo.txt'],
      unknown_items: ['unknown.txt'],
      can_cancel: true,
    },
  ],
  expandedTaskIds: ['task-preview-2'],
};

let state: TaskStoreState = initialState;
const listeners = new Set<Listener>();

function emitChange() {
  for (const listener of listeners) {
    listener();
  }
}

function setState(next: Partial<TaskStoreState>) {
  state = {
    ...state,
    ...next,
  };
  emitChange();
}

function subscribe(listener: Listener) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function toggleTaskExpanded(taskId: string) {
  setState({
    expandedTaskIds: state.expandedTaskIds.includes(taskId)
      ? state.expandedTaskIds.filter((id) => id !== taskId)
      : [...state.expandedTaskIds, taskId],
  });
}

function setTaskExpanded(taskId: string, expanded: boolean) {
  setState({
    expandedTaskIds: expanded
      ? Array.from(new Set([...state.expandedTaskIds, taskId]))
      : state.expandedTaskIds.filter((id) => id !== taskId),
  });
}

function setAllTasksExpanded(expanded: boolean) {
  setState({ expandedTaskIds: expanded ? state.tasks.map((task) => task.id) : [] });
}

export const taskStore: TaskStoreState & TaskStoreActions & { subscribe: typeof subscribe; getState: () => TaskStoreState; setState: typeof setState } = {
  get tasks() {
    return state.tasks;
  },
  get expandedTaskIds() {
    return state.expandedTaskIds;
  },
  setTasks: (tasks) => setState({ tasks }),
  toggleTaskExpanded,
  setTaskExpanded,
  setAllTasksExpanded,
  subscribe,
  getState: () => state,
  setState,
};

export function useTaskStore<T>(selector: (currentState: TaskStoreState & TaskStoreActions) => T): T {
  return useSyncExternalStore(subscribe, () => selector(taskStore), () => selector(taskStore));
}

export function resetTaskStore() {
  state = {
    tasks: initialState.tasks,
    expandedTaskIds: ['task-preview-2'],
  };
  emitChange();
}
