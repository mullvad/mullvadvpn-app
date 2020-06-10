import { sprintf } from 'sprintf-js';
import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { SystemNotificationProvider } from './notification';

export class ConnectedNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: TunnelState) {}

  public mayDisplay = () => this.context.state === 'connected';

  public getSystemNotification() {
    if (this.context.state === 'connected') {
      let message = messages.pgettext('notifications', 'Secured');
      const location = this.context.details.location?.hostname;
      if (location) {
        // TRANSLATORS: The message showed when a server has been connected to.
        // TRANSLATORS: Available placeholder:
        // TRANSLATORS: %(location) - name of the server location we're connected to (e.g. "se-got-003")
        message = sprintf(messages.pgettext('notifications', 'Connected to %(location)s'), {
          location,
        });
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
