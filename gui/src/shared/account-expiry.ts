import moment from 'moment';
import { sprintf } from 'sprintf-js';
import { messages } from './gettext';
import { capitalize } from './string-helpers';

type DateArgument = string | Date | moment.Moment;

export function hasExpired(expiry: DateArgument): boolean {
  return moment(expiry).isSameOrBefore(new Date());
}

export function formatDate(date: DateArgument, locale: string): string {
  return moment(date).locale(locale).format('lll');
}

export function formatDurationUntilExpiry(expiry: DateArgument, locale: string): string {
  const expiryMoment = moment(expiry).locale(locale);
  const daysDiff = expiryMoment.diff(new Date(), 'days');

  // Below three months we want to show the duration in days. Moments fromNow() method starts
  // measuring duration in months from 26 days and up.
  // https://momentjs.com/docs/#/displaying/fromnow/
  if (daysDiff >= 26 && daysDiff <= 90) {
    return sprintf(
      // TRANSLATORS: The remaining time left on the account measured in days.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(duration)s - The remaining time measured in days.
      messages.pgettext('account-expiry', '%(duration)s days'),
      { duration: daysDiff },
    );
  } else {
    return expiryMoment.fromNow(true);
  }
}

export function formatRemainingTime(
  expiry: DateArgument,
  locale: string,
  shouldCapitalizeFirstLetter?: boolean,
): string {
  const duration = formatDurationUntilExpiry(expiry, locale);

  const remaining = sprintf(
    // TRANSLATORS: The remaining time left on the account displayed across the app.
    // TRANSLATORS: Available placeholders:
    // TRANSLATORS: %(duration)s - a localized remaining time (in minutes, hours, or days) until the account expiry
    messages.pgettext('account-expiry', '%(duration)s left'),
    { duration },
  );

  return shouldCapitalizeFirstLetter ? capitalize(remaining) : remaining;
}
