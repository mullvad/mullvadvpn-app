// @flow

import log from 'electron-log';

/*
 * Used to execute a promise and retry it until it succeeds, with a dynamic delay between attempts.
 */
export default class DelayedPromiseRetries {
  _timeout: ?TimeoutID = null;
  _attemptCount = 0;
  _promise: () => Promise<void>;
  _delay: (number) => ?number;
  _name: string;

  constructor(promise: () => Promise<void>, delay: (number) => ?number, name?: string) {
    this._promise = promise;
    this._delay = delay;
    this._name = name || 'Promise';
  }

  stop() {
    if (this._timeout) {
      clearTimeout(this._timeout);
    }

    this._attemptCount = 0;
  }

  start() {
    this.stop();
    this._scheduleAttempt(0);
  }

  _scheduleAttempt(delay: number) {
    this._timeout = setTimeout(this._attempt, delay);
  }

  _attempt = async () => {
    this._timeout = null;

    try {
      await this._promise();
    } catch (error) {
      this._retry();
    }
  };

  _retry() {
    this._attemptCount += 1;

    const delay = this._delay(this._attemptCount);

    if (delay) {
      log.debug(
        `${this._name} attempt number ${this._attemptCount} failed, retrying in ${delay} ms.`,
      );
      this._scheduleAttempt(delay);
    } else {
      log.debug(`${this._name} attempt number ${this._attemptCount} failed, cancelling.`);
    }
  }
}
