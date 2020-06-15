import { useEffect, useMemo } from 'react';

export class Scheduler {
  private timer?: NodeJS.Timeout;
  private running = false;

  public schedule(action: () => void, delay = 0) {
    this.cancel();

    this.running = true;
    this.timer = global.setTimeout(() => {
      this.running = false;
      action();
    }, delay);
  }

  public cancel() {
    if (this.timer) {
      clearTimeout(this.timer);
      this.running = false;
    }
  }

  public get isRunning() {
    return this.running;
  }
}

export function useScheduler() {
  const closeScheduler = useMemo(() => new Scheduler(), []);

  useEffect(() => {
    return () => closeScheduler.cancel();
  }, []);

  return closeScheduler;
}
