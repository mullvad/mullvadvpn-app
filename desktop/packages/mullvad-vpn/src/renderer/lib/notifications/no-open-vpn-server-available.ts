import { sprintf } from 'sprintf-js';

import { strings } from '../../../shared/constants';
import {
  ErrorStateCause,
  TunnelParameterError,
  TunnelProtocol,
  TunnelState,
} from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { IRelayLocationCountryRedux } from '../../redux/settings/reducers';
import { RoutePath } from '../routes';

interface NoOpenVpnServerAvailableNotificationContext {
  tunnelProtocol: TunnelProtocol;
  tunnelState: TunnelState;
  relayLocations: IRelayLocationCountryRedux[];
}

export class NoOpenVpnServerAvailableNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: NoOpenVpnServerAvailableNotificationContext) {}

  public mayDisplay = () => {
    const { tunnelState, tunnelProtocol } = this.context;

    return (
      tunnelProtocol === 'openvpn' &&
      tunnelState.state === 'error' &&
      tunnelState.details.cause === ErrorStateCause.tunnelParameterError &&
      tunnelState.details.parameterError === TunnelParameterError.noMatchingRelay &&
      !this.anyOpenVpnLocationsEnabled()
    );
  };

  public getInAppNotification(): InAppNotification {
    const capitalizedOpenVpn = strings.openvpn.toUpperCase();
    return {
      indicator: 'error',
      title: sprintf(
        // TRANSLATORS: Notification title when there are no openVPN servers
        // TRANSLATORS: matching current settings.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(openVpn)s - Will be replaced with OPENVPN
        messages.pgettext('in-app-notifications', 'NO %(openVpn)s SERVER AVAILABLE'),
        { openVpn: capitalizedOpenVpn },
      ),
      subtitle: [
        {
          content: sprintf(
            // TRANSLATORS: First part of notification subtitle when there are no openVPN servers available.
            // TRANSLATORS: Will be followed by a link to VPN settings.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(openVpn)s - Will be replaced with OpenVPN
            messages.pgettext(
              'in-app-notifications',
              '%(openVpn)s support has ended. Please update the app or',
            ),
            { openVpn: strings.openvpn },
          ),
        },
        {
          content: sprintf(
            // TRANSLATORS: Link following the first part of the notification subtitle.
            // TRANSLATORS: Will navigate the user to the VPN settings.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
            messages.pgettext('in-app-notifications', 'change tunnel protocol to %(wireGuard)s.'),
            { wireGuard: strings.wireguard },
          ),
          action: {
            type: 'navigate-internal',
            link: {
              to: RoutePath.vpnSettings,
              'aria-label':
                // TRANSLATORS: Accessibility label for link to VPN settings where
                // TRANSLATORS: the user can change tunnel protocol.
                messages.pgettext('accessibility', 'Go to VPN settings to change tunnel protocol'),
            },
          },
        },
      ],
    };
  }

  private anyOpenVpnLocationsEnabled() {
    return this.context.relayLocations.some((location) => {
      return location.cities.some((city) => {
        return city.relays.some((relay) => {
          return relay.endpointType === 'openvpn' && relay.active;
        });
      });
    });
  }
}
