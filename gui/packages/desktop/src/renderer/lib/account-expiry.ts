import moment from 'moment';
import { sprintf } from 'sprintf-js';
import { pgettext } from '../../shared/gettext';

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
    const duration = this.expiry.fromNow(true);

    return sprintf(
      // TRANSLATORS: %(duration)s left
      pgettext('account-expiry', 'remaining-time'),
      { duration },
    );
  }
}
