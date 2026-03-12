import { sprintf } from 'sprintf-js';

import { InternalLink } from '../../renderer/components/InternalLink';
import { formatHtml } from '../../renderer/lib/html-formatter';
import { strings } from '../constants';
import { TunnelState } from '../daemon-rpc-types';
import { messages } from '../gettext';
import { RoutePath } from '../routes';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface LockdownModeNotificationContext {
  tunnelState: TunnelState;
  lockdownModeSetting: boolean;
  hasExcludedApps: boolean;
}

export class LockdownModeNotificationProvider
  implements InAppNotificationProvider, SystemNotificationProvider
{
  public constructor(private context: LockdownModeNotificationContext) {}

  public mayDisplay() {
    return (
      (this.context.tunnelState.state === 'disconnecting' && this.context.lockdownModeSetting) ||
      (this.context.tunnelState.state === 'disconnected' && this.context.tunnelState.lockedDown)
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
      messages.pgettext('in-app-notifications', '<a>%(lockdownModeSettingName)s</a> is enabled.'),
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

    const formattedSubtitle = formatHtml(subtitle, {
      a: (value) => (
        <InternalLink
          to={RoutePath.vpnSettings}
          variant="labelTinySemiBold"
          locationState={{
            options: [
              {
                type: 'scroll-to-anchor',
                id: 'lockdown-mode-setting',
              },
            ],
          }}>
          <InternalLink.Text>{value}</InternalLink.Text>
        </InternalLink>
      ),
    });

    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
      subtitle: formattedSubtitle,
    };
  }
}
