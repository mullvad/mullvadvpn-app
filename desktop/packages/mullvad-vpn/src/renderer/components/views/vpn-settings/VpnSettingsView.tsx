import { messages } from '../../../../shared/gettext';
import { AllowLanSetting } from '../../../features/lan-sharing/components';
import {
  EnableIpv6Setting,
  LockdownModeSetting,
  QuantumResistantSetting,
} from '../../../features/tunnel/components';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
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
  AntiCensorshipListItem,
  AutoConnectSetting,
  AutoStartSetting,
  CustomDnsSettings,
  DnsBlockerSettings,
  IpOverrideSettings,
  IpVersionSetting,
  KillSwitchSetting,
  MtuSetting,
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
                    <AutoStartSetting />
                    <AutoConnectSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <AllowLanSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <DnsBlockerSettings />
                    <CustomDnsSettings />
                  </SettingsGroup>

                  <SettingsGroup>
                    <EnableIpv6Setting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <KillSwitchSetting />
                    <LockdownModeSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <AntiCensorshipListItem />
                  </SettingsGroup>

                  <SettingsGroup>
                    <QuantumResistantSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <IpVersionSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <MtuSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <IpOverrideSettings />
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
