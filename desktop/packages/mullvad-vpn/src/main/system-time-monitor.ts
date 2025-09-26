const INTERVAL = 1000;

// This functions monitors the system clock for changes, such as NTP corrections or user manually
// changing the time. It probably has a lot of false positives, e.g. after suspend. And it only
// checks once a second so the event will be a bit delayed.
export function systemTimeMonitor(listener: () => void) {
  let prevDate = Date.now();
  setInterval(() => {
    const now = Date.now();
    if (Math.abs(now - prevDate - INTERVAL) > 500) {
      listener();
    }
    prevDate = now;
  }, INTERVAL);
}
