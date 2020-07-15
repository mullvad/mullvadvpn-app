import moment from 'moment';
import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import { formatDurationUntilExpiry, formatRemainingTime, hasExpired } from '../account-expiry';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationProvider,
} from './notification';

interface CloseToAccountExpiryNotificationContext {
  accountExpiry: string;
  locale: string;
}

export class CloseToAccountExpiryNotificationProvider
  implements InAppNotificationProvider, SystemNotificationProvider {
  public constructor(private context: CloseToAccountExpiryNotificationContext) {}

  public mayDisplay() {
    const willHaveExpiredInThreeDays = moment(this.context.accountExpiry).isSameOrBefore(
      moment().add(3, 'days'),
    );

    return !hasExpired(this.context.accountExpiry) && willHaveExpiredInThreeDays;
  }

  public getSystemNotification(): SystemNotification {
    const message = sprintf(
      // TRANSLATORS: The system notification displayed to the user when the account credit is close to expiry.
      // TRANSLATORS: Available placeholder:
      // TRANSLATORS: %(duration)s - remaining time, e.g. "2 days"
      messages.pgettext('notifications', 'Account credit expires in %(duration)s'),
      {
        duration: formatDurationUntilExpiry(this.context.accountExpiry, this.context.locale),
      },
    );

    return {
      message,
      critical: true,
      action: {
        type: 'open-url',
        url: links.purchase,
        withAuth: true,
        text: messages.pgettext('notifications', 'Buy more'),
      },
    };
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'ACCOUNT CREDIT EXPIRES SOON'),
      subtitle: formatRemainingTime(this.context.accountExpiry, this.context.locale, true),
      action: { type: 'open-url', url: links.purchase, withAuth: true },
    };
  }
}
