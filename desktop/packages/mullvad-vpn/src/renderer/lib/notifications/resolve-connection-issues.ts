import { messages } from '../../../shared/gettext';
import type { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../../../shared/routes';

interface ResolveConnectionIssuesNotificationContext {
  hasSettingsMigrations: boolean;
  displayedSettingsMigrations: boolean;
  connectionBlocked: boolean;
}

export class ResolveConnectionIssuesNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: ResolveConnectionIssuesNotificationContext) {}

  public mayDisplay = () => {
    return (
      this.context.hasSettingsMigrations &&
      !this.context.displayedSettingsMigrations &&
      this.context.connectionBlocked
    );
  };

  public getInAppNotification(): InAppNotification {
    const title =
      // TRANSLATORS: Title for notification that is displayed when user has a blocked connection due to settings migration.
      messages.pgettext('in-app-notifications', 'RESOLVE CONNECTION ISSUES');
    const subtitle =
      // TRANSLATORS: Subtitle for notification that is displayed when user has a blocked connection due to settings migration.
      messages.pgettext(
        'in-app-notifications',
        'Some of your settings have been migrated, please review the changes and resolve any issues.',
      );
    const link = messages.pgettext('in-app-notifications', 'Click here to read more');
    return {
      indicator: 'error',
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
