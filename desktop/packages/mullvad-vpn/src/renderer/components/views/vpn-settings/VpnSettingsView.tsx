import { sprintf } from 'sprintf-js';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { useHistory } from '../../../lib/history';
import { useTunnelProtocol } from '../../../lib/relay-settings-hooks';
import { RelaySettingsRedux } from '../../../redux/settings/reducers';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import CustomDnsSettings from '../../CustomDnsSettings';
import { BackAction } from '../../KeyboardNavigation';
import {
  Layout,
  SettingsContainer,
  SettingsContent,
  SettingsGroup,
  SettingsStack,
} from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationListItem } from '../../NavigationListItem';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import {
  AllowLan,
  AutoConnect,
  AutoStart,
  DnsBlockers,
  EnableIpv6,
  KillSwitchInfo,
  LockdownMode,
  TunnelProtocolSetting,
} from './components';

export function VpnSettingsView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('vpn-settings-view', 'VPN settings')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('vpn-settings-view', 'VPN settings')}</HeaderTitle>
              </SettingsHeader>

              <SettingsContent>
                <SettingsStack>
                  <SettingsGroup>
                    <AutoStart />
                    <AutoConnect />
                  </SettingsGroup>

                  <SettingsGroup>
                    <AllowLan />
                  </SettingsGroup>

                  <SettingsGroup>
                    <DnsBlockers />
                  </SettingsGroup>

                  <SettingsGroup>
                    <EnableIpv6 />
                  </SettingsGroup>

                  <SettingsGroup>
                    <KillSwitchInfo />
                    <LockdownMode />
                  </SettingsGroup>

                  <SettingsGroup>
                    <TunnelProtocolSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <WireguardSettingsButton />
                    <OpenVpnSettingsButton />
                  </SettingsGroup>

                  <SettingsGroup>
                    <CustomDnsSettings />
                  </SettingsGroup>

                  <SettingsGroup>
                    <IpOverrideButton />
                  </SettingsGroup>
                </SettingsStack>
              </SettingsContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function mapRelaySettingsToProtocol(relaySettings: RelaySettingsRedux) {
  if ('normal' in relaySettings) {
    const { tunnelProtocol } = relaySettings.normal;
    return tunnelProtocol;
    // since the GUI doesn't display custom settings, just display the default ones.
    // If the user sets any settings, then those will be applied.
  } else if ('customTunnelEndpoint' in relaySettings) {
    return undefined;
  } else {
    throw new Error('Unknown type of relay settings.');
  }
}

function WireguardSettingsButton() {
  const tunnelProtocol = useSelector((state) =>
    mapRelaySettingsToProtocol(state.settings.relaySettings),
  );

  return (
    <NavigationListItem to={RoutePath.wireguardSettings} disabled={tunnelProtocol === 'openvpn'}>
      <NavigationListItem.Label>
        {sprintf(
          // TRANSLATORS: %(wireguard)s will be replaced with the string "WireGuard"
          messages.pgettext('vpn-settings-view', '%(wireguard)s settings'),
          { wireguard: strings.wireguard },
        )}
      </NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

function OpenVpnSettingsButton() {
  const tunnelProtocol = useTunnelProtocol();

  return (
    <NavigationListItem to={RoutePath.openVpnSettings} disabled={tunnelProtocol === 'wireguard'}>
      <NavigationListItem.Label>
        {sprintf(
          // TRANSLATORS: %(openvpn)s will be replaced with the string "OpenVPN"
          messages.pgettext('vpn-settings-view', '%(openvpn)s settings'),
          { openvpn: strings.openvpn },
        )}
      </NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

function IpOverrideButton() {
  return (
    <NavigationListItem to={RoutePath.settingsImport}>
      <NavigationListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Server IP override')}
      </NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}
