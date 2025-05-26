import { messages } from '../../shared/gettext';
import { AppVersionInfoSuggestedUpgrade } from '../daemon-rpc-types';
import { getDownloadUrl } from '../version';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface UnsupportedVersionNotificationContext {
  supported: boolean;
  consistent: boolean;
  suggestedUpgrade?: AppVersionInfoSuggestedUpgrade;
  suggestedIsBeta?: boolean;
}

export class UnsupportedVersionNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider
{
  public constructor(private context: UnsupportedVersionNotificationContext) {}

  public mayDisplay() {
    return this.context.consistent && !this.context.supported;
  }

  public getSystemNotification(): SystemNotification {
    return {
      message: this.getMessage(),
      category: SystemNotificationCategory.newVersion,
      severity: SystemNotificationSeverityType.high,
      action: {
        type: 'navigate-external',
        link: {
          text: messages.pgettext('notifications', 'Upgrade'),
          to: getDownloadUrl(this.context.suggestedIsBeta ?? false),
        },
      },
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'UNSUPPORTED VERSION'),
      subtitle: this.getMessage(),
      action: {
        type: 'navigate-external',
        link: {
          to: getDownloadUrl(this.context.suggestedIsBeta ?? false),
        },
      },
    };
  }

  private getMessage(): string {
    // TRANSLATORS: The in-app banner and system notification which are displayed to the user when the running app becomes unsupported.
    return messages.pgettext(
      'notifications',
      'Your privacy might be at risk with this unsupported app version. Please update now.',
    );
  }
}
