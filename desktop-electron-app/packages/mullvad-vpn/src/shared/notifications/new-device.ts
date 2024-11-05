import { sprintf } from 'sprintf-js';

import { messages } from '../../shared/gettext';
import { capitalizeEveryWord } from '../string-helpers';
import { InAppNotification, InAppNotificationProvider } from './notification';

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
        messages.pgettext(
          'in-app-notifications',
          'Welcome, this device is now called <b>%(deviceName)s</b>. For more details see the info button in Account.',
        ),
        { deviceName: capitalizeEveryWord(this.context.deviceName) },
      ),
      action: { type: 'close', close: this.context.close },
    };
  }
}
