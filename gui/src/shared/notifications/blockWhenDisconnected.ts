import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  InAppNotification,
  InAppNotificationIndicatorType,
  NotificationProvider,
  SystemNotification,
} from './notification';

interface BlockWhenDisconnectedNotificationContext {
  tunnelState: TunnelState;
  blockWhenDisconnected: boolean;
}

export class BlockWhenDisconnectedNotificationProvider
  extends NotificationProvider<BlockWhenDisconnectedNotificationContext>
  implements InAppNotification, SystemNotification {
  public get visible() {
    return (
      (this.context.tunnelState.state === 'disconnecting' ||
        this.context.tunnelState.state === 'disconnected') &&
      this.context.blockWhenDisconnected
    );
  }

  public message = messages.pgettext('notifications', 'Blocking internet');

  public critical = false;

  public indicator: InAppNotificationIndicatorType = 'error';

  public title = messages.pgettext('in-app-notifications', 'BLOCKING INTERNET');

  public body = messages.pgettext('in-app-notifications', '"Always require VPN" is enabled.');
}
