export class Scheduler {
  private timer?: NodeJS.Timeout;

  public schedule(action: () => void, delay: number) {
    this.cancel();
    this.timer = global.setTimeout(action, delay);
  }

  public cancel() {
    if (this.timer) {
      clearTimeout(this.timer);
    }
  }
}
