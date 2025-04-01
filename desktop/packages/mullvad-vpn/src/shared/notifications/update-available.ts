import { sprintf } from 'sprintf-js';

import { messages } from '../../shared/gettext';
import { AppVersionInfoSuggestedUpgrade } from '../daemon-rpc-types';
import { getDownloadUrl } from '../version';
import {
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface UpdateAvailableNotificationContext {
  suggestedUpgrade?: AppVersionInfoSuggestedUpgrade;
  suggestedIsBeta?: boolean;
}

export class UpdateAvailableNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: UpdateAvailableNotificationContext) {}

  public mayDisplay() {
    return this.context.suggestedUpgrade?.version ? true : false;
  }

  public getSystemNotification(): SystemNotification {
    return {
      message: this.systemMessage(),
      category: SystemNotificationCategory.newVersion,
      severity: SystemNotificationSeverityType.medium,
      action: {
        type: 'open-url',
        url: getDownloadUrl(this.context.suggestedIsBeta ?? false),
        text: messages.pgettext('notifications', 'Upgrade'),
      },
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
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
        { version: this.context.suggestedUpgrade?.version },
      );
    } else {
      return messages.pgettext(
        'notifications',
        'Update available. Install the latest app version to stay up to date',
      );
    }
  }
}
