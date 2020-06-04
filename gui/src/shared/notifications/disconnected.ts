import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { SystemNotificationProvider } from './notification';

export class DisconnectedNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: TunnelState) {}

  public mayDisplay = () => this.context.state === 'disconnected';

  public getSystemNotification() {
    return {
      message: messages.pgettext('notifications', 'Unsecured'),
      critical: false,
    };
  }
}
