import { sprintf } from 'sprintf-js';
import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { NotificationProvider, SystemNotification } from './notification';

interface ConnectedNotificationContext {
  tunnelState: TunnelState;
}

export class ConnectedNotificationProvider
  extends NotificationProvider<ConnectedNotificationContext>
  implements SystemNotification {
  public get visible() {
    return this.context.tunnelState.state === 'connected';
  }

  public get message() {
    if (this.context.tunnelState.state !== 'connected') {
      throw Error('ConnectedNotificationProvider message getter called without being connected');
    }

    const location = this.context.tunnelState.details.location?.hostname;
    if (location) {
      // TRANSLATORS: The message showed when a server has been connected to.
      // TRANSLATORS: Available placeholder:
      // TRANSLATORS: %(location) - name of the server location we're connected to (e.g. "se-got-003")
      return sprintf(messages.pgettext('notifications', 'Connected to %(location)s'), { location });
    } else {
      return messages.pgettext('notifications', 'Secured');
    }
  }

  public critical = false;
}
