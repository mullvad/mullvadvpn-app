import { sprintf } from 'sprintf-js';

import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
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
      title: this.context.suggestedIsBeta
        ? messages.pgettext('in-app-notifications', 'BETA UPDATE AVAILABLE')
        : messages.pgettext('in-app-notifications', 'UPDATE AVAILABLE'),
      subtitle: this.inAppMessage(),
      action: {
        type: 'open-url',
        url: this.context.suggestedIsBeta ? links.betaDownload : links.download,
      },
    };
  }

  public getSystemNotification(): SystemNotification {
    return {
      message: this.systemMessage(),
      category: SystemNotificationCategory.newVersion,
      severity: SystemNotificationSeverityType.medium,
      action: {
        type: 'open-url',
        url: this.context.suggestedIsBeta ? links.betaDownload : links.download,
        text: messages.pgettext('notifications', 'Upgrade'),
      },
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }

  private inAppMessage(): string {
    if (this.context.suggestedIsBeta) {
      return sprintf(
        // TRANSLATORS: The in-app banner displayed to the user when the app beta update is
        // TRANSLATORS: available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(version)s - The version number of the new beta version.
        messages.pgettext('in-app-notifications', 'Try out the newest beta version (%(version)s).'),
        { version: this.context.suggestedUpgrade },
      );
    } else {
      // TRANSLATORS: The in-app banner displayed to the user when the app update is available.
      return messages.pgettext(
        'in-app-notifications',
        'Install the latest app version to stay up to date.',
      );
    }
  }

  private systemMessage(): string {
    if (this.context.suggestedIsBeta) {
      return sprintf(
        // TRANSLATORS: The system notification that notifies the user when a beta update is
        // TRANSLATORS: available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(version)s - The version number of the new beta version.
        messages.pgettext(
          'notifications',
          'Beta update available. Try out the newest beta version (%(version)s).',
        ),
        { version: this.context.suggestedUpgrade },
      );
    } else {
      return messages.pgettext(
        'notifications',
        'Update available. Install the latest app version to stay up to date',
      );
    }
  }
}
