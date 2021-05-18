import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { SystemNotificationProvider } from './notification';

export class ReconnectingNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: TunnelState) {}

  public mayDisplay() {
    return this.context.state === 'disconnecting' && this.context.details === 'reconnect';
  }

  public getSystemNotification() {
    return {
      message: messages.pgettext('notifications', 'Reconnecting'),
      critical: false,
    };
  }
}
