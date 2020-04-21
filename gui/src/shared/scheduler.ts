export class Scheduler {
  private timer?: number;

  public schedule(action: () => void, delay: number) {
    this.cancel();
    this.timer = setTimeout(action, delay);
  }

  public cancel() {
    clearTimeout(this.timer);
  }
}
