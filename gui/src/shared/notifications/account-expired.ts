import { links } from '../../config.json';
import { hasExpired } from '../account-expiry';
import { TunnelState } from '../daemon-rpc-types';
import { messages } from '../gettext';
import {
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface AccountExpiredNotificaitonContext {
  accountExpiry: string;
  tunnelState: TunnelState;
}

export class AccountExpiredNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: AccountExpiredNotificaitonContext) {}

  public mayDisplay() {
    // Only show when disconnected since the error state handles this if the connection is closed
    // due to account expiry.
    return (
      this.context.tunnelState.state === 'disconnected' && hasExpired(this.context.accountExpiry)
    );
  }

  public getSystemNotification(): SystemNotification {
    return {
      message: messages.pgettext('notifications', 'Account is out of time'),
      category: SystemNotificationCategory.expiry,
      severity: SystemNotificationSeverityType.high,
      presentOnce: { value: true, name: this.constructor.name },
      action: {
        type: 'open-url',
        url: links.purchase,
        withAuth: true,
        text: messages.pgettext('notifications', 'Buy more'),
      },
    };
  }
}
