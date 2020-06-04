import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  InAppNotification,
  InAppNotificationIndicatorType,
  NotificationProvider,
  SystemNotification,
} from './notification';

interface ReconnectingNotificationContext {
  tunnelState: TunnelState;
}

export class ReconnectingNotificationProvider
  extends NotificationProvider<ReconnectingNotificationContext>
  implements SystemNotification, InAppNotification {
  public get visible() {
    return (
      this.context.tunnelState.state === 'disconnecting' &&
      this.context.tunnelState.details === 'reconnect'
    );
  }

  public message = messages.pgettext('notifications', 'Reconnecting');

  public critical = false;

  public indicator: InAppNotificationIndicatorType = 'error';

  public title = messages.pgettext('in-app-notifications', 'BLOCKING INTERNET');
}
