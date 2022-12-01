import { sprintf } from 'sprintf-js';

import { strings } from '../../config.json';
import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface BlockWhenDisconnectedNotificationContext {
  tunnelState: TunnelState;
  blockWhenDisconnected: boolean;
  hasExcludedApps: boolean;
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

  public getSystemNotification(): SystemNotification {
    const message = messages.pgettext('notifications', 'Lockdown mode active, connection blocked');

    return {
      message,
      severity: SystemNotificationSeverityType.info,
      category: SystemNotificationCategory.tunnelState,
    };
  }

  public getInAppNotification(): InAppNotification {
    const lockdownModeSettingName = messages.pgettext('vpn-settings-view', 'Lockdown mode');
    let subtitle = sprintf(
      messages.pgettext('in-app-notifications', '"%(lockdownModeSettingName)s" is enabled.'),
      { lockdownModeSettingName },
    );
    if (this.context.hasExcludedApps) {
      subtitle = `${subtitle} ${sprintf(
        messages.pgettext(
          'notifications',
          'The apps excluded with %(splitTunneling)s might not work properly right now.',
        ),
        { splitTunneling: strings.splitTunneling.toLowerCase() },
      )}`;
    }

    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
      subtitle,
    };
  }
}
