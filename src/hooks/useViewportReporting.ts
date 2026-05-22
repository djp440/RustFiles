import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import type { SchedulerReportAck, VisibleRange } from '../api/tauri';

interface UseViewportReportingOptions {
  activeTabId: string;
  interactionEpoch: number;
  lastInputAtMs: number | null;
  visibleRange: VisibleRange | null;
  reportWhenRangeMissing?: boolean;
}

interface UseViewportReportingResult {
  scrollRef: RefObject<HTMLDivElement>;
  handleScroll: () => void;
  setVisibleRange: (visibleRange: VisibleRange | null) => void;
  latestAck: SchedulerReportAck | null;
  isScrolling: boolean;
}

function createViewportSignal(
  activeTabId: string,
  interactionEpoch: number,
  lastInputAtMs: number | null,
  visibleRange: VisibleRange | null,
  isScrolling: boolean,
) {
  return {
    active_tab_id: activeTabId,
    visible_range: visibleRange,
    last_input_at_ms: lastInputAtMs,
    is_scrolling: isScrolling,
    interaction_epoch: interactionEpoch,
  };
}

function getSchedulerReportingTarget(): {
  __RUSTFILES_REPORT_VIEWPORT_STATE__?: (
    signal: ReturnType<typeof createViewportSignal>,
  ) => Promise<SchedulerReportAck>;
} {
  return globalThis as unknown as {
    __RUSTFILES_REPORT_VIEWPORT_STATE__?: (
      signal: ReturnType<typeof createViewportSignal>,
    ) => Promise<SchedulerReportAck>;
  };
}

export function useViewportReporting({
  activeTabId,
  interactionEpoch,
  lastInputAtMs,
  visibleRange,
  reportWhenRangeMissing = false,
}: UseViewportReportingOptions): UseViewportReportingResult {
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const [trackedVisibleRange, setTrackedVisibleRange] = useState<VisibleRange | null>(visibleRange);
  const [isScrolling, setIsScrolling] = useState(false);
  const [latestAck, setLatestAck] = useState<SchedulerReportAck | null>(null);
  const scrollResetTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastReportKeyRef = useRef<string>('');
  const previousVisibleRangeRef = useRef<VisibleRange | null>(visibleRange);

  useEffect(() => {
    const previousVisibleRange = previousVisibleRangeRef.current;
    if (
      previousVisibleRange?.start_index === visibleRange?.start_index &&
      previousVisibleRange?.end_index === visibleRange?.end_index
    ) {
      return;
    }

    previousVisibleRangeRef.current = visibleRange;
    setTrackedVisibleRange(visibleRange);
  }, [visibleRange]);

  useEffect(() => {
    return () => {
      if (scrollResetTimerRef.current !== null) {
        clearTimeout(scrollResetTimerRef.current);
      }
    };
  }, []);

  const handleScroll = useCallback(() => {
    setIsScrolling(true);

    if (scrollResetTimerRef.current !== null) {
      clearTimeout(scrollResetTimerRef.current);
    }

    scrollResetTimerRef.current = setTimeout(() => {
      setIsScrolling(false);
      scrollResetTimerRef.current = null;
    }, 180);
  }, []);

  useEffect(() => {
    if (trackedVisibleRange === null && !reportWhenRangeMissing) {
      return;
    }

    const signal = createViewportSignal(
      activeTabId,
      interactionEpoch,
      lastInputAtMs,
      trackedVisibleRange,
      isScrolling,
    );
    const reportKey = JSON.stringify(signal);

    if (reportKey === lastReportKeyRef.current) {
      return;
    }

    lastReportKeyRef.current = reportKey;

    const reportViewportState = getSchedulerReportingTarget().__RUSTFILES_REPORT_VIEWPORT_STATE__;
    if (!reportViewportState) {
      setLatestAck({} as SchedulerReportAck);
      return;
    }

    void reportViewportState(signal).then((ack) => {
      setLatestAck(ack);
    });
  }, [
    activeTabId,
    interactionEpoch,
    isScrolling,
    lastInputAtMs,
    reportWhenRangeMissing,
    trackedVisibleRange,
  ]);

  return {
    scrollRef,
    handleScroll,
    setVisibleRange: setTrackedVisibleRange,
    latestAck,
    isScrolling,
  };
}
