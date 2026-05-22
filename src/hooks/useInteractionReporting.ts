import { useCallback, useEffect, useRef, useState } from 'react';
import type { SchedulerReportAck, VisibleRange } from '../api/tauri';

interface UseInteractionReportingOptions {
  activeTabId: string;
}

interface InteractionReportOptions {
  isScrolling?: boolean;
  visibleRange?: VisibleRange | null;
}

interface UseInteractionReportingResult {
  interactionEpoch: number;
  lastInputAtMs: number | null;
  reportInteraction: (options?: InteractionReportOptions) => Promise<SchedulerReportAck>;
}

function createInteractionSignal(
  activeTabId: string,
  interactionEpoch: number,
  lastInputAtMs: number,
  options: InteractionReportOptions = {},
) {
  return {
    active_tab_id: activeTabId,
    visible_range: options.visibleRange ?? null,
    last_input_at_ms: lastInputAtMs,
    is_scrolling: options.isScrolling ?? false,
    interaction_epoch: interactionEpoch,
  };
}

function getSchedulerReportingTarget(): {
  __RUSTFILES_REPORT_INTERACTION_STATE__?: (
    signal: ReturnType<typeof createInteractionSignal>,
  ) => Promise<SchedulerReportAck>;
} {
  return globalThis as unknown as {
    __RUSTFILES_REPORT_INTERACTION_STATE__?: (
      signal: ReturnType<typeof createInteractionSignal>,
    ) => Promise<SchedulerReportAck>;
  };
}

export function useInteractionReporting({
  activeTabId,
}: UseInteractionReportingOptions): UseInteractionReportingResult {
  const interactionEpochRef = useRef(0);
  const [interactionEpoch, setInteractionEpoch] = useState(0);
  const [lastInputAtMs, setLastInputAtMs] = useState<number | null>(null);

  useEffect(() => {
    interactionEpochRef.current = 0;
    setInteractionEpoch(0);
    setLastInputAtMs(null);
  }, [activeTabId]);

  const reportInteraction = useCallback(
    async (options: InteractionReportOptions = {}) => {
      const nextInteractionEpoch = interactionEpochRef.current + 1;
      interactionEpochRef.current = nextInteractionEpoch;

      const nextLastInputAtMs = Date.now();
      setInteractionEpoch(nextInteractionEpoch);
      setLastInputAtMs(nextLastInputAtMs);

      const reportInteractionState = getSchedulerReportingTarget().__RUSTFILES_REPORT_INTERACTION_STATE__;
      if (!reportInteractionState) {
        return Promise.resolve({} as SchedulerReportAck);
      }

      return reportInteractionState(
        createInteractionSignal(activeTabId, nextInteractionEpoch, nextLastInputAtMs, options),
      );
    },
    [activeTabId],
  );

  return {
    interactionEpoch,
    lastInputAtMs,
    reportInteraction,
  };
}
