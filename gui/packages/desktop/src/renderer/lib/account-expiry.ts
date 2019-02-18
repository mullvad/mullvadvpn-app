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
      // TRANSLATORS: The remaining time left on the account displayed across the app.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(duration)s - a localized remaining time (in minutes, hours, or days) until the account expiry
      pgettext('account-expiry', '%(duration)s left'),
      { duration },
    );
  }
}
