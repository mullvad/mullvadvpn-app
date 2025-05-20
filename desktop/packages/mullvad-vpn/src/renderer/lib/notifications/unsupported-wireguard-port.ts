import { sprintf } from 'sprintf-js';

import { strings } from '../../../shared/constants';
import { TunnelProtocol } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../../../shared/routes';
import { isInRanges } from '../../../shared/utils';
import { IConnectionReduxState } from '../../redux/connection/reducers';
import { RelaySettingsRedux } from '../../redux/settings/reducers';

interface UnsupportedWireGuardPortNotificationContext {
  connection: IConnectionReduxState;
  tunnelProtocol: TunnelProtocol;
  relaySettings: RelaySettingsRedux;
  allowedPortRanges: [number, number][];
}

export class UnsupportedWireGuardPortNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: UnsupportedWireGuardPortNotificationContext) {}

  public mayDisplay = () => {
    const { connection, tunnelProtocol, relaySettings, allowedPortRanges } = this.context;
    if (tunnelProtocol === 'wireguard' && connection.status.state === 'error') {
      if ('normal' in relaySettings) {
        const { port } = relaySettings.normal.wireguard;
        if (port !== 'any' && !isInRanges(port, allowedPortRanges)) return true;
      }
    }
    return false;
  };

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
      subtitle: [
        {
          content: sprintf(
            // TRANSLATORS: Notification subtitle indicating the user is using an unsupported port for WireGuard.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
            messages.pgettext(
              'in-app-notifications',
              'The selected %(wireGuard)s port is not supported, please change it under ',
            ),
            { wireGuard: strings.wireguard },
          ),
        },
        {
          content: sprintf(
            // TRANSLATORS: Link in notication to go to WireGuard settings.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
            messages.pgettext('in-app-notifications', '%(wireGuard)s settings.'),
            { wireGuard: strings.wireguard },
          ),
          action: {
            type: 'navigate-internal',
            link: {
              to: RoutePath.wireguardSettings,
              'aria-label': sprintf(
                // TRANSLATORS: Accessibility label for link to wireguard settings.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
                messages.pgettext('accessibility', 'Go to %(wireGuard)s settings.'),
                { wireGuard: strings.wireguard },
              ),
            },
          },
        },
      ],
    };
  }
}
