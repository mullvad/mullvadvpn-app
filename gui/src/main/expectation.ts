export default class Expectation {
  private fulfilled = false;
  private timeout: NodeJS.Timeout;

  constructor(private handler: () => void, timeout = 2000) {
    this.timeout = global.setTimeout(() => {
      this.fulfill();
    }, timeout);
  }

  public fulfill() {
    if (this.fulfilled) {
      return;
    }

    this.fulfilled = true;
    global.clearTimeout(this.timeout);
    this.handler();
  }
}
