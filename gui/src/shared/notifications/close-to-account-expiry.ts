import { sprintf } from 'sprintf-js';

import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import { closeToExpiry, formatRemainingTime } from '../account-expiry';
import { formatRelativeDate } from '../date-helper';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface CloseToAccountExpiryNotificationContext {
  accountExpiry: string;
  locale: string;
}

export class CloseToAccountExpiryNotificationProvider
  implements InAppNotificationProvider, SystemNotificationProvider {
  public constructor(private context: CloseToAccountExpiryNotificationContext) {}

  public mayDisplay = () => closeToExpiry(this.context.accountExpiry);

  public getSystemNotification(): SystemNotification {
    const message = sprintf(
      // TRANSLATORS: The system notification displayed to the user when the account credit is close to expiry.
      // TRANSLATORS: Available placeholder:
      // TRANSLATORS: %(duration)s - remaining time, e.g. "2 days"
      messages.pgettext(
        'notifications',
        'Account credit expires in %(duration)s. Buy more credit.',
      ),
      {
        duration: formatRelativeDate(new Date(), this.context.accountExpiry),
      },
    );

    return {
      message,
      category: SystemNotificationCategory.expiry,
      severity: SystemNotificationSeverityType.medium,
      action: {
        type: 'open-url',
        url: links.purchase,
        withAuth: true,
        text: messages.pgettext('notifications', 'Buy more'),
      },
    };
  }

  public getInAppNotification(): InAppNotification {
    const subtitle = sprintf(
      messages.pgettext('in-app-notifications', '%(duration)s. Buy more credit.'),
      { duration: formatRemainingTime(this.context.accountExpiry, true) },
    );

    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'ACCOUNT CREDIT EXPIRES SOON'),
      subtitle,
      action: { type: 'open-url', url: links.purchase, withAuth: true },
    };
  }
}
