import { sprintf } from 'sprintf-js';

import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

export class ConnectedNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: TunnelState) {}

  public mayDisplay = () => this.context.state === 'connected';

  public getSystemNotification(): SystemNotification | undefined {
    if (this.context.state === 'connected') {
      let message = messages.pgettext('notifications', 'Connected');
      const location = this.context.details.location?.hostname;
      if (location) {
        message = sprintf(
          // TRANSLATORS: The message showed when a server has been connected to.
          // TRANSLATORS: Available placeholder:
          // TRANSLATORS: %(location) - name of the server location we're connected to (e.g. "se-got-003")
          messages.pgettext('notifications', 'Connected to %(location)s'),
          {
            location,
          },
        );
      }

      return {
        message,
        severity: SystemNotificationSeverityType.info,
        category: SystemNotificationCategory.tunnelState,
      };
    } else {
      return undefined;
    }
  }
}
