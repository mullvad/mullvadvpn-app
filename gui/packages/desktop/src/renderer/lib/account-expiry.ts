import moment from 'moment';

export default class AccountExpiry {
  _expiry: moment.Moment;

  constructor(expiry: string) {
    this._expiry = moment(expiry);
  }

  hasExpired(): boolean {
    return this.willHaveExpiredIn(moment());
  }

  willHaveExpiredIn(time: moment.Moment): boolean {
    return this._expiry.isSameOrBefore(time);
  }

  remainingTime(): string {
    return this._expiry.fromNow(true) + ' left';
  }
}
