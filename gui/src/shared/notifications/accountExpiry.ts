import moment from 'moment';
import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import AccountExpiry from '../account-expiry';
import {
  InAppNotification,
  NotificationAction,
  InAppNotificationIndicatorType,
  NotificationProvider,
  SystemNotification,
} from './notification';

interface AccountExpiryContext {
  accountExpiry?: AccountExpiry;
  tooSoon?: boolean;
}

export class AccountExpiryNotificationProvider extends NotificationProvider<AccountExpiryContext>
  implements InAppNotification, SystemNotification {
  public get visible() {
    return (
      this.context.accountExpiry !== undefined &&
      !this.context.accountExpiry.hasExpired() &&
      this.context.accountExpiry.willHaveExpiredAt(moment().add(3, 'days').toDate()) &&
      !this.context.tooSoon
    );
  }

  public action: NotificationAction = { type: 'open-url', url: links.purchase, withAuth: true };

  public get message() {
    // TRANSLATORS: The system notification displayed to the user when the account credit is close to expiry.
    // TRANSLATORS: Available placeholder:
    // TRANSLATORS: %(duration)s - remaining time, e.g. "2 days"
    return sprintf(messages.pgettext('notifications', 'Account credit expires in %(duration)s'), {
      duration: this.context.accountExpiry!.remainingTime(),
    });
  }

  public critical = true;

  public indicator: InAppNotificationIndicatorType = 'warning';

  public title = messages.pgettext('in-app-notifications', 'ACCOUNT CREDIT EXPIRES SOON');

  public get body() {
    return capitalizeFirstLetter(this.context.accountExpiry!.remainingTime());
  }
}

function capitalizeFirstLetter(inputString: string): string {
  return inputString.charAt(0).toUpperCase() + inputString.slice(1);
}
