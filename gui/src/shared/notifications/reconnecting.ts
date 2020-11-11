import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotificationProvider,
} from './notification';

export class ReconnectingNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider {
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

  public getInAppNotification(): InAppNotification {
    return {
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
    };
  }
}
