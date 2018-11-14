// @flow

/*
 * Used to calculate the time to wait before reconnecting to the daemon.
 * It uses a linear backoff function that goes from 500ms to 3000ms.
 */
export default class ReconnectionBackoff {
  _attempt = 0;

  attempt(handler: () => void) {
    setTimeout(handler, this._getIncreasedBackoff());
  }

  reset() {
    this._attempt = 0;
  }

  _getIncreasedBackoff() {
    if (this._attempt < 6) {
      this._attempt++;
    }
    return this._attempt * 500;
  }
}
