import { render, screen } from '@testing-library/react';
import { within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { resetTaskStore, taskStore } from '../stores/tasks';
import TaskPanel from '../components/tasks/TaskPanel';

describe('TaskPanel', () => {
  beforeEach(() => {
    resetTaskStore();
  });

  it('shows a compact summary without forcing expansion', () => {
    taskStore.setState({
      tasks: [
        {
          id: 'task-1',
          kind: 'copy_items',
          status: 'running',
          message: 'Copying 2 items',
          completed_items: ['a.txt'],
          incomplete_items: ['b.txt'],
          unknown_items: [],
          can_cancel: true,
        },
      ],
      expandedTaskIds: [],
    });

    render(<TaskPanel />);

    expect(screen.getByRole('button', { name: /tasks/i })).toBeInTheDocument();
    expect(screen.getByText('Copying 2 items')).toBeInTheDocument();
    expect(screen.queryByText('Completed')).not.toBeInTheDocument();
  });

  it('shows expanded details and conflict/cancel states', () => {
    taskStore.setState({
      tasks: [
        {
          id: 'task-2',
          kind: 'move_items',
          status: 'waiting_for_conflict_decision',
          message: 'Waiting for conflict decision',
          completed_items: ['done.txt'],
          incomplete_items: ['todo.txt'],
          unknown_items: ['unknown.txt'],
          can_cancel: true,
        },
        {
          id: 'task-3',
          kind: 'delete_to_recycle_bin',
          status: 'cancelling',
          message: 'Cancelling task',
          completed_items: [],
          incomplete_items: ['pending.txt'],
          unknown_items: [],
          can_cancel: false,
        },
      ],
      expandedTaskIds: ['task-2', 'task-3'],
    });

    render(<TaskPanel />);

    const panel = screen.getByRole('region', { name: 'Task panel' });
    expect(within(panel).getAllByText('Waiting for conflict decision').length).toBeGreaterThan(0);
    expect(within(panel).getByText('Cancelling')).toBeInTheDocument();
  });

  it('renders failed and partially completed summaries clearly', () => {
    taskStore.setState({
      tasks: [
        {
          id: 'task-4',
          kind: 'copy_items',
          status: 'failed',
          message: 'Copy failed',
          completed_items: ['kept.txt'],
          incomplete_items: ['failed.txt'],
          unknown_items: ['maybe.txt'],
          can_cancel: false,
        },
        {
          id: 'task-5',
          kind: 'move_items',
          status: 'partially_completed',
          message: 'Partially completed',
          completed_items: ['moved.txt'],
          incomplete_items: ['left.txt'],
          unknown_items: ['unknown.txt'],
          can_cancel: false,
        },
      ],
      expandedTaskIds: ['task-4', 'task-5'],
    });

    render(<TaskPanel />);

    const panel = screen.getByRole('region', { name: 'Task panel' });
    expect(within(panel).getByText('Copy failed')).toBeInTheDocument();
    expect(within(panel).getAllByText('Partially completed').length).toBeGreaterThan(0);
    expect(within(panel).getByText('Failed')).toBeInTheDocument();
  });
});
