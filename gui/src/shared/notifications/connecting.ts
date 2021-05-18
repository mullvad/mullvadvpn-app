import { sprintf } from 'sprintf-js';
import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { SystemNotificationProvider } from './notification';

interface ConnectingNotificationContext {
  tunnelState: TunnelState;
  reconnecting?: boolean;
}

export class ConnectingNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: ConnectingNotificationContext) {}

  public mayDisplay() {
    return this.context.tunnelState.state === 'connecting' && !this.context.reconnecting;
  }

  public getSystemNotification() {
    if (this.context.tunnelState.state === 'connecting') {
      let message = messages.pgettext('notifications', 'Connecting');
      const location = this.context.tunnelState.details?.location?.hostname;
      if (location) {
        message = sprintf(
          // TRANSLATORS: The message showed when a server is being connected to.
          // TRANSLATORS: Available placeholder:
          // TRANSLATORS: %(location) - name of the server location we're connecting to (e.g. "se-got-003")
          messages.pgettext('notifications', 'Connecting to %(location)s'),
          {
            location,
          },
        );
      }

      return {
        message,
        critical: false,
      };
    } else {
      return undefined;
    }
  }
}
