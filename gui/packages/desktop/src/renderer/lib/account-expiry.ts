import moment from 'moment';

export default class AccountExpiry {
  private expiry: moment.Moment;

  constructor(expiry: string) {
    this.expiry = moment(expiry);
  }

  public hasExpired(): boolean {
    return this.willHaveExpiredIn(moment());
  }

  public willHaveExpiredIn(time: moment.Moment): boolean {
    return this.expiry.isSameOrBefore(time);
  }

  public remainingTime(): string {
    return this.expiry.fromNow(true) + ' left';
  }
}
