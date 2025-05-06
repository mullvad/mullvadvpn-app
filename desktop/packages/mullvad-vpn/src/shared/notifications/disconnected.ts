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
}

export class DisconnectedNotificationProvider implements SystemNotificationProvider {
  public constructor(private context: DisconnectedNotificationContext) {}

  public mayDisplay = () =>
    this.context.tunnelState.state === 'disconnected' && !this.context.tunnelState.lockedDown;

  public getSystemNotification(): SystemNotification | undefined {
    return {
      message: messages.pgettext('notifications', 'Disconnected and unsecure'),
      severity: SystemNotificationSeverityType.info,
      category: SystemNotificationCategory.tunnelState,
    };
  }
}
