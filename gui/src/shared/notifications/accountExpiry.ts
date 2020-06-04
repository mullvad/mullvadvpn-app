import moment from 'moment';
import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import AccountExpiry from '../account-expiry';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationProvider,
} from './notification';

interface AccountExpiryContext {
  accountExpiry: AccountExpiry;
  tooSoon?: boolean;
}

export class AccountExpiryNotificationProvider
  implements InAppNotificationProvider, SystemNotificationProvider {
  public constructor(private context: AccountExpiryContext) {}

  public mayDisplay() {
    return (
      !this.context.accountExpiry.hasExpired() &&
      this.context.accountExpiry.willHaveExpiredAt(moment().add(3, 'days').toDate()) &&
      !this.context.tooSoon
    );
  }

  public getSystemNotification(): SystemNotification {
    const message = sprintf(
      // TRANSLATORS: The system notification displayed to the user when the account credit is close to expiry.
      // TRANSLATORS: Available placeholder:
      // TRANSLATORS: %(duration)s - remaining time, e.g. "2 days"
      messages.pgettext('notifications', 'Account credit expires in %(duration)s'),
      {
        duration: this.context.accountExpiry.remainingTime(),
      },
    );

    return {
      message,
      critical: true,
      action: { type: 'open-url', url: links.purchase, withAuth: true },
    };
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'ACCOUNT CREDIT EXPIRES SOON'),
      subtitle: this.context.accountExpiry.remainingTime(true),
      action: { type: 'open-url', url: links.purchase, withAuth: true },
    };
  }
}
