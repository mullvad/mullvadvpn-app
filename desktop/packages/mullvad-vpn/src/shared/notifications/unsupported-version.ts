import { messages } from '../../shared/gettext';
import { RoutePath } from '../../shared/routes';
import { AppVersionInfoSuggestedUpgrade } from '../daemon-rpc-types';
import { getDownloadUrl } from '../version';
import {
  InAppNotification,
  InAppNotificationAction,
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
      action: this.context.suggestedUpgrade
        ? {
            type: 'navigate-internal',
            link: {
              to: RoutePath.appUpgrade,
            },
          }
        : {
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
      subtitle: [
        {
          content:
            // TRANSLATORS: The in-app banner which is displayed to the user when the running app becomes unsupported.
            messages.pgettext(
              'notifications',
              'Your privacy might be at risk with this unsupported app version.',
            ),
        },
        {
          content:
            // TRANSLATORS: A link in the in-app banner to encourage the user to update the app.
            // TRANSLATORS: The in-app banner is is displayed to the user when the running app becomes unsupported.
            messages.pgettext('notifications', 'Please click here to update now'),
          action: this.getInAppNotificationAction(),
        },
      ],
    };
  }

  private getInAppNotificationAction(): InAppNotificationAction {
    if (this.context.suggestedUpgrade) {
      if (process.platform !== 'linux') {
        return {
          type: 'navigate-internal',
          link: {
            to: RoutePath.appUpgrade,
          },
        };
      }
    }

    return {
      type: 'navigate-external',
      link: {
        to: getDownloadUrl(this.context.suggestedIsBeta ?? false),
      },
    };
  }

  private getMessage(): string {
    // TRANSLATORS: The system notification which is displayed to the user when the running app becomes unsupported.
    return messages.pgettext(
      'notifications',
      'Your privacy might be at risk with this unsupported app version. Please update now.',
    );
  }
}
