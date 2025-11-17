import { sprintf } from 'sprintf-js';

import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../../../shared/routes';
import { InternalLink } from '../../components/InternalLink';
import { formatHtml } from '../html-formatter';
import { formatDeviceName } from '../utils';

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
      subtitle: formatHtml(
        sprintf(
          // TRANSLATORS: Notification text when a new device has been created.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: - %(deviceName)s: Name of created device.
          messages.pgettext(
            'in-app-notifications',
            'This device is now named <em>%(deviceName)s</em>. See more under <a>Manage devices</a> in Account.',
          ),
          { deviceName: formatDeviceName(this.context.deviceName) },
        ),
        {
          a: (value) => (
            <InternalLink variant="labelTinySemiBold" to={RoutePath.manageDevices}>
              <InternalLink.Text>{value}</InternalLink.Text>
            </InternalLink>
          ),
        },
      ),
      action: { type: 'close', close: this.context.close },
    };
  }
}
