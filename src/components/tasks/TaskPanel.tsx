import { useTaskStore } from '../../stores/tasks';
import GlassSurface from '../surfaces/GlassSurface';

function describeStatus(status: string) {
  switch (status) {
    case 'waiting_for_conflict_decision':
      return 'Waiting for conflict decision';
    case 'cancelling':
      return 'Cancelling';
    case 'failed':
      return 'Failed';
    case 'partially_completed':
      return 'Partially completed';
    case 'completed':
      return 'Completed';
    case 'cancelled':
      return 'Cancelled';
    case 'running':
      return 'Running';
    case 'validating':
      return 'Validating';
    case 'queued':
    default:
      return 'Queued';
  }
}

function TaskPanel() {
  const tasks = useTaskStore((state) => state.tasks);
  const expandedTaskIds = useTaskStore((state) => state.expandedTaskIds);
  const setAllTasksExpanded = useTaskStore((state) => state.setAllTasksExpanded);
  const hasTasks = tasks.length > 0;
  const totalCompletedItems = tasks.reduce((count, task) => count + task.completed_items.length, 0);
  const isExpanded = tasks.length > 0 && expandedTaskIds.length === tasks.length;

  return (
    <GlassSurface
      variant="content"
      role="region"
      aria-label="Task panel"
      style={{
        display: 'grid',
        gap: 12,
        padding: 12,
        borderBottom: '1px solid var(--border-subtle)',
        maxHeight: 240,
        overflow: 'auto',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
        <div>
          <strong style={{ color: 'var(--text-primary)' }}>Tasks</strong>
          <div style={{ fontSize: 12, color: 'var(--text-secondary)' }}>{hasTasks ? `${tasks.length} active task(s)` : 'No tasks'}</div>
        </div>
        <button type="button" onClick={() => setAllTasksExpanded(!isExpanded)}>
          {isExpanded ? 'Collapse tasks' : 'Expand tasks'}
        </button>
      </div>

      <div style={{ fontSize: 12, color: 'var(--text-tertiary)' }}>Completed items: {totalCompletedItems}</div>

      {tasks.length === 0 ? (
        <div style={{ fontSize: 13, color: 'var(--text-secondary)' }}>No tasks running.</div>
      ) : (
        <div style={{ display: 'grid', gap: 8 }}>
          {tasks.map((task) => {
            const expanded = expandedTaskIds.includes(task.id);

            return (
              <section
                key={task.id}
                style={{
                  padding: 10,
                  borderRadius: 'var(--radius-sm)',
                  background: 'var(--surface-floating)',
                  border: '1px solid var(--border-subtle)',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', gap: 12 }}>
                  <div style={{ minWidth: 0 }}>
                    <div style={{ color: 'var(--text-primary)', fontSize: 13, fontWeight: 600 }}>{task.kind}</div>
                    <div style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{task.message ?? describeStatus(task.status)}</div>
                  </div>
                  <div style={{ color: 'var(--text-tertiary)', fontSize: 12 }}>{describeStatus(task.status)}</div>
                </div>

                {expanded ? (
                  <div style={{ display: 'grid', gap: 6, marginTop: 8, fontSize: 12, color: 'var(--text-secondary)' }}>
                    {task.kind === 'copy_items' || task.kind === 'move_items' ? (
                      <div>
                        {(task as any).progressTotal > 0 ? (
                          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                            <progress max={(task as any).progressTotal} value={(task as any).progressCurrent} style={{ flex: 1 }} />
                            <span>{Math.round(((task as any).progressCurrent / (task as any).progressTotal) * 100)}%</span>
                          </div>
                        ) : null}
                      </div>
                    ) : null}
                    <div>Completed items: {task.completed_items.length ? task.completed_items.join(', ') : 'None'}</div>
                    <div>Incomplete items: {task.incomplete_items.length ? task.incomplete_items.join(', ') : 'None'}</div>
                    <div>Unknown items: {task.unknown_items.length ? task.unknown_items.join(', ') : 'None'}</div>
                    <div>Can cancel: {task.can_cancel ? 'Yes' : 'No'}</div>
                  </div>
                ) : null}
              </section>
            );
          })}
        </div>
      )}
    </GlassSurface>
  );
}

export default TaskPanel;
