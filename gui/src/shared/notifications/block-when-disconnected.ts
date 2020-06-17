import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotificationProvider,
} from './notification';

interface BlockWhenDisconnectedNotificationContext {
  tunnelState: TunnelState;
  blockWhenDisconnected: boolean;
}

export class BlockWhenDisconnectedNotificationProvider
  implements InAppNotificationProvider, SystemNotificationProvider {
  public constructor(private context: BlockWhenDisconnectedNotificationContext) {}

  public mayDisplay() {
    return (
      (this.context.tunnelState.state === 'disconnecting' ||
        this.context.tunnelState.state === 'disconnected') &&
      this.context.blockWhenDisconnected
    );
  }

  public getSystemNotification() {
    return {
      message: messages.pgettext('notifications', 'Blocking internet'),
      critical: false,
    };
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
      subtitle: messages.pgettext('in-app-notifications', '"Always require VPN" is enabled.'),
    };
  }
}
