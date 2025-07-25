import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';

import { strings, urls } from '../../../../shared/constants';
import { TunnelProtocol } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import log from '../../../../shared/logging';
import { RoutePath } from '../../../../shared/routes';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useHistory } from '../../../lib/history';
import { useTunnelProtocol } from '../../../lib/relay-settings-hooks';
import { RelaySettingsRedux } from '../../../redux/settings/reducers';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { AriaDescription, AriaInputGroup } from '../../AriaGroup';
import * as Cell from '../../cell';
import Selector, { SelectorItem } from '../../cell/Selector';
import CustomDnsSettings from '../../CustomDnsSettings';
import { ExternalLink } from '../../ExternalLink';
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

function TunnelProtocolSetting() {
  const tunnelProtocol = useTunnelProtocol();

  const relaySettingsUpdater = useRelaySettingsUpdater();

  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);
  const openVpnDisabled = daita || multihop || quantumResistant;

  const featuresToDisableForOpenVpn = [];
  if (daita) {
    featuresToDisableForOpenVpn.push(strings.daita);
  }
  if (multihop) {
    featuresToDisableForOpenVpn.push(messages.pgettext('wireguard-settings-view', 'Multihop'));
  }
  if (quantumResistant) {
    featuresToDisableForOpenVpn.push(
      messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel'),
    );
  }

  const setTunnelProtocol = useCallback(
    async (tunnelProtocol: TunnelProtocol) => {
      try {
        await relaySettingsUpdater((settings) => ({
          ...settings,
          tunnelProtocol,
        }));
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update tunnel protocol constraints', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  const tunnelProtocolItems: Array<SelectorItem<TunnelProtocol>> = useMemo(
    () => [
      {
        label: strings.wireguard,
        value: 'wireguard',
      },
      {
        label: strings.openvpn,
        value: 'openvpn',
        disabled: openVpnDisabled,
      },
    ],
    [openVpnDisabled],
  );

  return (
    <AriaInputGroup>
      <Selector
        title={messages.pgettext('vpn-settings-view', 'Tunnel protocol')}
        items={tunnelProtocolItems}
        value={tunnelProtocol}
        onSelect={setTunnelProtocol}
      />
      {openVpnDisabled && (
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                messages.pgettext(
                  'vpn-settings-view',
                  'To select %(openvpn)s, please disable these settings: %(featureList)s.',
                ),
                { openvpn: strings.openvpn, featureList: featuresToDisableForOpenVpn.join(', ') },
              )}
            </Cell.CellFooterText>
          </AriaDescription>
        </Cell.CellFooter>
      )}
      {tunnelProtocol === 'openvpn' && (
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                // TRANSLATORS: Footer text for tunnel protocol selector when OpenVPN is selected.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(openvpn)s - Will be replaced with OpenVPN
                messages.pgettext(
                  'vpn-settings-view',
                  'Attention: We are removing support for %(openVpn)s.',
                ),
                { openVpn: strings.openvpn },
              )}{' '}
            </Cell.CellFooterText>
          </AriaDescription>
          <ExternalLink variant="labelTiny" to={urls.removingOpenVpnBlog}>
            <ExternalLink.Text>
              {sprintf(
                // TRANSLATORS: Link in tunnel protocol selector footer to blog post
                // TRANSLATORS: about OpenVPN support ending.
                messages.pgettext('vpn-settings-view', 'Read more'),
              )}
            </ExternalLink.Text>
            <ExternalLink.Icon icon="external" size="small" />
          </ExternalLink>
        </Cell.CellFooter>
      )}
    </AriaInputGroup>
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
