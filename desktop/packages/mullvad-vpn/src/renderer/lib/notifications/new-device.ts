import { sprintf } from 'sprintf-js';

import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { capitalizeEveryWord } from '../../../shared/string-helpers';

interface NewDeviceNotificationContext {
  shouldDisplay: boolean;
  deviceName: string;
  close: () => void;
}

export class NewDeviceNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: NewDeviceNotificationContext) {}

  public mayDisplay = () => this.context.shouldDisplay;

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'success',
      title: messages.pgettext('in-app-notifications', 'NEW DEVICE CREATED'),
      subtitle: sprintf(
        // TRANSLATORS: Notification text when a new device has been created.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: - %(deviceName)s: Name of created device.
        messages.pgettext(
          'in-app-notifications',
          'This device is now named <em>%(deviceName)s</em>. See more under "Manage devices" in Account.',
        ),
        { deviceName: capitalizeEveryWord(this.context.deviceName) },
      ),
      action: { type: 'close', close: this.context.close },
    };
  }
}
