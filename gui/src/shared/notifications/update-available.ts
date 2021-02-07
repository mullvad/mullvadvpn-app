import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationProvider,
} from './notification';

interface UpdateAvailableNotificationContext {
  suggestedUpgrade?: string;
  suggestedIsBeta?: boolean;
}

export class UpdateAvailableNotificationProvider
  implements InAppNotificationProvider, SystemNotificationProvider {
  public constructor(private context: UpdateAvailableNotificationContext) {}

  public mayDisplay() {
    return this.context.suggestedUpgrade ? true : false;
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'UPDATE AVAILABLE'),
      // TRANSLATORS: The in-app banner displayed to the user when the app update is available.
      subtitle: messages.pgettext(
        'in-app-notifications',
        'Install the latest app version to stay up to date.',
      ),
      action: {
        type: 'open-url',
        url: this.context.suggestedIsBeta ? links.betaDownload : links.download,
      },
    };
  }

  public getSystemNotification(): SystemNotification {
    return {
      message: messages.pgettext(
        'notifications',
        'Update available. Install the latest app version to stay up to date',
      ),
      critical: false,
      action: {
        type: 'open-url',
        url: this.context.suggestedIsBeta ? links.betaDownload : links.download,
        text: messages.pgettext('notifications', 'Upgrade'),
      },
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }
}
