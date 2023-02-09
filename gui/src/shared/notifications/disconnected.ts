import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface DisconnectedNotificationContext {
  tunnelState: TunnelState;
  blockWhenDisconnected: boolean;
}

export class DisconnectedNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: DisconnectedNotificationContext) {}

  public mayDisplay = () =>
    this.context.tunnelState.state === 'disconnected' && !this.context.blockWhenDisconnected;

  public getSystemNotification(): SystemNotification | undefined {
    return {
      message: messages.pgettext('notifications', 'Disconnected and unsecure'),
      severity: SystemNotificationSeverityType.info,
      category: SystemNotificationCategory.tunnelState,
    };
  }
}
