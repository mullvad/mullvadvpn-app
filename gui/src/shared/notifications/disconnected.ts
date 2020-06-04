import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { NotificationProvider, SystemNotification } from './notification';

interface DisconnectedNotificationContext {
  tunnelState: TunnelState;
}

export class DisconnectedNotificationProvider
  extends NotificationProvider<DisconnectedNotificationContext>
  implements SystemNotification {
  public get visible() {
    return this.context.tunnelState.state === 'disconnected';
  }

  public message = messages.pgettext('notifications', 'Unsecured');

  public critical = false;
}
