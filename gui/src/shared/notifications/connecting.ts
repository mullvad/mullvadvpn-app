import { sprintf } from 'sprintf-js';
import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  InAppNotification,
  InAppNotificationIndicatorType,
  NotificationProvider,
  SystemNotification,
} from './notification';

interface ConnectingNotificationContext {
  tunnelState: TunnelState;
  reconnecting?: boolean;
}

export class ConnectingNotificationProvider
  extends NotificationProvider<ConnectingNotificationContext>
  implements SystemNotification, InAppNotification {
  public get visible() {
    return this.context.tunnelState.state === 'connecting' && !this.context.reconnecting;
  }

  public get message() {
    if (this.context.tunnelState.state !== 'connecting') {
      throw Error('ConnectingNotificationProvider message getter called without being connecting');
    }

    const location = this.context.tunnelState.details?.location?.hostname;
    if (location) {
      // TRANSLATORS: The message showed when a server is being connected to.
      // TRANSLATORS: Available placeholder:
      // TRANSLATORS: %(location) - name of the server location we're connecting to (e.g. "se-got-003")
      return sprintf(messages.pgettext('notifications', 'Connecting to %(location)s'), {
        location,
      });
    } else {
      return messages.pgettext('notifications', 'Connecting');
    }
  }

  public critical = false;

  public indicator: InAppNotificationIndicatorType = 'error';

  public title = messages.pgettext('in-app-notifications', 'BLOCKING INTERNET');
}
