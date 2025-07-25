import { messages } from '../../../../shared/gettext';
import { useHistory } from '../../../lib/history';
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
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import {
  AllowLan,
  AutoConnect,
  AutoStart,
  DnsBlockers,
  EnableIpv6,
  IpOverrideButton,
  KillSwitchInfo,
  LockdownMode,
  OpenVpnSettingsButton,
  TunnelProtocolSetting,
  WireguardSettingsButton,
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
