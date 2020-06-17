import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from './notification';

interface UpdateAvailableNotificationContext {
  current: string;
  nextUpgrade: string | null;
}

export class UpdateAvailableNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: UpdateAvailableNotificationContext) {}

  public mayDisplay() {
    return this.context.nextUpgrade !== null && this.context.nextUpgrade !== this.context.current;
  }

  public getInAppNotification(): InAppNotification {
    const subtitle = sprintf(
      // TRANSLATORS: The in-app banner displayed to the user when the app update is available.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(version)s - the newest available version of the app
      messages.pgettext(
        'in-app-notifications',
        'Install Mullvad VPN (%(version)s) to stay up to date',
      ),
      { version: this.context.nextUpgrade },
    );

    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'UPDATE AVAILABLE'),
      subtitle,
      action: { type: 'open-url', url: links.download },
    };
  }
}
