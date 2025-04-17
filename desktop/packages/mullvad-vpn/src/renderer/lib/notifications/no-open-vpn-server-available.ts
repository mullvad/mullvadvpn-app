import { sprintf } from 'sprintf-js';

import { strings } from '../../../shared/constants';
import {
  ErrorStateCause,
  TunnelParameterError,
  TunnelProtocol,
} from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import {
  InAppNotification,
  InAppNotificationProvider,
  InAppNotificationSubtitle,
} from '../../../shared/notifications';
import { IConnectionReduxState } from '../../redux/connection/reducers';
import {
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../redux/settings/reducers';
import { RoutePath } from '../routes';

interface NoOpenVpnServerAvailableNotificationContext {
  connection: IConnectionReduxState;
  tunnelProtocol: TunnelProtocol;
  relayLocations: IRelayLocationCountryRedux[];
}

export class NoOpenVpnServerAvailableNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: NoOpenVpnServerAvailableNotificationContext) {}

  public mayDisplay = () => {
    const { connection, tunnelProtocol } = this.context;

    const hasNoEnabledOpenVpnRelays = !this.anyOpenVpnLocationsEnabled();
    const isSelectedRelayOpenVpn = this.isSelectedRelayOpenVpn();
    const isTunnelProtocolOpenVpn = tunnelProtocol === 'openvpn';
    const hasNoMatchingRelay =
      connection.status.state === 'error' &&
      connection.status.details.cause === ErrorStateCause.tunnelParameterError &&
      connection.status.details.parameterError === TunnelParameterError.noMatchingRelay;

    return (
      isTunnelProtocolOpenVpn &&
      hasNoMatchingRelay &&
      (isSelectedRelayOpenVpn || hasNoEnabledOpenVpnRelays)
    );
  };

  public getInAppNotification(): InAppNotification {
    let title: string = '';
    const subtitle: InAppNotificationSubtitle[] = [];
    const capitalizedOpenVpn = strings.openvpn.toUpperCase();

    if (this.anyOpenVpnLocationsEnabled()) {
      title = sprintf(
        // TRANSLATORS: Notification title when there are no openVPN servers
        // TRANSLATORS: matching current settings.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(openVpn)s - Will be replaced with OPENVPN
        messages.pgettext('in-app-notifications', 'NO %(openVpn)s SERVER AVAILABLE'),
        { openVpn: capitalizedOpenVpn },
      );
      subtitle.push({
        content: sprintf(
          // TRANSLATORS: First part of notification subtitle when there are no openVPN servers
          // TRANSLATORS: matching current settings. Will be followed by a link to VPN settings.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(openVpn)s - Will be replaced with OpenVPN
          messages.pgettext(
            'in-app-notifications',
            '%(openVpn)s support is ending. Switch location or',
          ),
          { openVpn: strings.openvpn },
        ),
      });
    } else {
      title = sprintf(
        // TRANSLATORS: Notification title when there are no openVPN servers available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(openVpn)s - Will be replaced with OPENVPN
        messages.pgettext('in-app-notifications', 'NO %(openVpn)s SERVERS AVAILABLE'),
        { openVpn: capitalizedOpenVpn },
      );
      subtitle.push({
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
      });
    }
    subtitle.push({
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
    });

    return {
      indicator: 'error',
      title,
      subtitle,
    };
  }

  private isSelectedRelayOpenVpn() {
    const selectedRelay = this.getSelectedRelay();

    if (selectedRelay) {
      return selectedRelay.endpointType === 'openvpn' || selectedRelay.endpointType === 'bridge';
    }

    return false;
  }

  private getSelectedRelay() {
    const selectedRelay = this.context.relayLocations.reduce<IRelayLocationRelayRedux | undefined>(
      (selectedRelay, location) => {
        const isSelectedRelayCountry = location.name === this.context.connection.country;
        if (isSelectedRelayCountry) {
          const relayCity = location.cities.find(
            (city) => city.name === this.context.connection.city,
          );

          if (relayCity) {
            const relay = relayCity.relays.find(
              (relay) => relay.hostname === this.context.connection.hostname,
            );

            if (relay) {
              return relay;
            }
          }
        }

        return selectedRelay;
      },
      undefined,
    );

    return selectedRelay;
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
