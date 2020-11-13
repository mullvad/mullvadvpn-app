import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { SystemNotificationProvider } from './notification';

interface DisconnectedNotificationContext {
  tunnelState: TunnelState;
  blockWhenDisconnected: boolean;
}

export class DisconnectedNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: DisconnectedNotificationContext) {}

  public mayDisplay = () =>
    this.context.tunnelState.state === 'disconnected' && !this.context.blockWhenDisconnected;

  public getSystemNotification() {
    return {
      message: messages.pgettext('notifications', 'Disconnected and unsecured'),
      critical: false,
    };
  }
}
