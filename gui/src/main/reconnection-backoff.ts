/*
 * Used to calculate the time to wait before reconnecting to the daemon.
 * It uses a linear backoff function that goes from 500ms to 3000ms.
 */
export default class ReconnectionBackoff {
  private attemptValue = 0;

  public attempt(handler: () => void) {
    setTimeout(handler, this.getIncreasedBackoff());
  }

  public reset() {
    this.attemptValue = 0;
  }

  private getIncreasedBackoff() {
    if (this.attemptValue < 6) {
      this.attemptValue++;
    }
    return this.attemptValue * 500;
  }
}
