import { messages } from '../../../../shared/gettext';
import { AutoConnectSetting, AutoStartSetting } from '../../../features/client/components';
import { AllowLanSetting } from '../../../features/lan-sharing/components';
import {
  EnableIpv6Setting,
  LockdownModeSetting,
  QuantumResistantSetting,
} from '../../../features/tunnel/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer, SettingsContent } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import {
  AntiCensorshipListItem,
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
                <FlexColumn gap="medium">
                  <FlexColumn>
                    <AutoStartSetting />
                    <AutoConnectSetting />
                  </FlexColumn>

                  <AllowLanSetting />

                  <FlexColumn>
                    <DnsBlockerSettings />
                    <CustomDnsSettings />
                  </FlexColumn>

                  <EnableIpv6Setting />
                  <KillSwitchSetting />
                  <LockdownModeSetting />
                  <AntiCensorshipListItem />
                  <QuantumResistantSetting />
                  <IpVersionSetting />
                  <MtuSetting />
                  <IpOverrideSettings />
                </FlexColumn>
              </SettingsContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
