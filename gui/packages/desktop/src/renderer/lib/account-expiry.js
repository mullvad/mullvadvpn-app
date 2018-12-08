// @flow

import moment from 'moment';

export default class AccountExpiry {
  _expiry: ?moment;

  constructor(expiry: ?string) {
    this._expiry = expiry ? moment(expiry) : null;
  }

  hasExpired(): boolean {
    return this.willHaveExpiredIn(moment());
  }

  willHaveExpiredIn(time: moment): boolean {
    return this._expiry ? this._expiry.isSameOrBefore(time) : false;
  }

  remainingTime(): string {
    return this._expiry ? this._expiry.fromNow(true) + ' left' : '';
  }
}
