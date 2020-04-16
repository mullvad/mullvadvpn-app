export class Scheduler {
  private timer?: NodeJS.Timeout;

  public schedule(action: () => void, delay = 0) {
    this.cancel();
    this.timer = global.setTimeout(action, delay);
  }

  public cancel() {
    if (this.timer) {
      clearTimeout(this.timer);
    }
  }
}
