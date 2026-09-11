import { messages } from '../../../shared/gettext';
import type { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../../../shared/routes';

interface SettingsMigratedNotificationContext {
  hasSettingsMigrations: boolean;
  dismissedSettingsMigrations: boolean;
  displayedSettingsMigrations: boolean;
  close: () => void;
}

export class SettingsMigratedNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: SettingsMigratedNotificationContext) {}

  public mayDisplay = () => {
    return (
      this.context.hasSettingsMigrations &&
      !this.context.dismissedSettingsMigrations &&
      !this.context.displayedSettingsMigrations
    );
  };

  public getInAppNotification(): InAppNotification {
    const title =
      // TRANSLATORS: Title for notification that is displayed when user has had their settings migrated.
      messages.pgettext('in-app-notifications', 'SETTINGS MIGRATED');
    const subtitle =
      // TRANSLATORS: Subtitle for notification that is displayed when user has had their settings migrated.
      messages.pgettext(
        'in-app-notifications',
        'Some of your settings have been migrated, please review the changes.',
      );
    const link = messages.pgettext('in-app-notifications', 'Click here to read more');
    return {
      indicator: 'warning',
      action: { type: 'close', close: this.context.close },
      title,
      subtitle: [
        {
          content: subtitle,
        },
        {
          content: link,
          action: {
            type: 'navigate-internal',
            link: {
              to: RoutePath.migratedSettings,
            },
          },
        },
      ],
    };
  }
}
