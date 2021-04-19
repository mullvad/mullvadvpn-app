import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { InAppNotification, InAppNotificationProvider } from './notification';

interface BlockWhenDisconnectedNotificationContext {
  tunnelState: TunnelState;
  blockWhenDisconnected: boolean;
  hasExcludedApps: boolean;
}

export class BlockWhenDisconnectedNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: BlockWhenDisconnectedNotificationContext) {}

  public mayDisplay() {
    return (
      (this.context.tunnelState.state === 'disconnecting' ||
        this.context.tunnelState.state === 'disconnected') &&
      this.context.blockWhenDisconnected
    );
  }

  public getInAppNotification(): InAppNotification {
    let subtitle = messages.pgettext('in-app-notifications', '"Always require VPN" is enabled.');
    if (this.context.hasExcludedApps) {
      subtitle = `${subtitle} ${messages.pgettext(
        'notifications',
        'The apps excluded with split tunneling might not work properly right now.',
      )}`;
    }

    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
      subtitle,
    };
  }
}
