import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { systemTimeMonitor } from '../../src/main/system-time-monitor';

describe('IAccountData cache', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should notify when system clock changes', () => {
    const systemTimeListener = vi.fn();

    vi.setSystemTime(new Date('2025-01-01'));
    systemTimeMonitor(systemTimeListener);
    vi.setSystemTime(new Date('2025-01-02'));
    vi.advanceTimersByTime(1001);
    vi.setSystemTime(new Date('2025-01-01'));
    vi.advanceTimersByTime(1900);

    expect(systemTimeListener).toHaveBeenCalledTimes(2);
  });
});
