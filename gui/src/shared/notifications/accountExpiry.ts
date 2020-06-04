import moment from 'moment';
import { messages } from '../../shared/gettext';
import AccountExpiry from '../account-expiry';
import { NotificationIndicatorType } from './notification';

export const accountExpiry = {
  condition: (accountExpiry?: AccountExpiry) =>
    accountExpiry &&
    !accountExpiry.hasExpired() &&
    accountExpiry.willHaveExpiredAt(moment().add(3, 'days').toDate()),
  systemNotification: {
    // TRANSLATORS: The system notification displayed to the user when the account credit is close to expiry.
    // TRANSLATORS: Available placeholder:
    // TRANSLATORS: %(duration)s - remaining time, e.g. "2 days"
    message: messages.pgettext('notifications', 'Account credit expires in %(duration)s'),
    important: true,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'ACCOUNT CREDIT EXPIRES SOON'),
    body: messages.pgettext('in-app-notifications', '%(duration)s left'),
  },
};
