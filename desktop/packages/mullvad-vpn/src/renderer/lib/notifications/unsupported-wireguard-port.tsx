import { sprintf } from 'sprintf-js';

import { strings } from '../../../shared/constants';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../../../shared/routes';
import { isInRanges } from '../../../shared/utils';
import { InternalLink } from '../../components/InternalLink';
import { IConnectionReduxState } from '../../redux/connection/reducers';
import { RelaySettingsRedux } from '../../redux/settings/reducers';
import { formatHtml } from '../html-formatter';

interface UnsupportedWireGuardPortNotificationContext {
  connection: IConnectionReduxState;
  relaySettings: RelaySettingsRedux;
  allowedPortRanges: [number, number][];
}

const transformerMap = {
  a: (value: string) => (
    <InternalLink
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for link to VPN settings where
        // TRANSLATORS: the user can change WireGuard port.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
        messages.pgettext('accessibility', 'Go to VPN settings to change %(wireGuard)s port'),
        { wireGuard: strings.wireguard },
      )}
      variant="labelTinySemiBold"
      to={RoutePath.vpnSettings}>
      <InternalLink.Text>{value}</InternalLink.Text>
    </InternalLink>
  ),
};

export class UnsupportedWireGuardPortNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: UnsupportedWireGuardPortNotificationContext) {}

  public mayDisplay = () => {
    const { connection, relaySettings, allowedPortRanges } = this.context;
    if (connection.status.state === 'error') {
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
          key: 'in-app-notifications-unsupported-wireguard-port',
          content: formatHtml(
            sprintf(
              // TRANSLATORS: Notification subtitle indicating the user is using an unsupported port for WireGuard.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
              messages.pgettext(
                'in-app-notifications',
                'The selected %(wireGuard)s port is not supported, please <a>change it under VPN settings.</a>',
              ),
              { wireGuard: strings.wireguard },
            ),
            transformerMap,
          ),
        },
      ],
    };
  }
}
