import { messages } from '../../../../shared/gettext';
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
  AllowLanSetting,
  AutoConnectSetting,
  AutoStartSetting,
  CensorshipCircumventionListItem,
  CustomDnsSettings,
  DnsBlockerSettings,
  EnableIpv6Setting,
  IpOverrideSettings,
  KillSwitchSetting,
  LockdownModeSetting,
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
                  </SettingsGroup>

                  <SettingsGroup>
                    <EnableIpv6Setting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <KillSwitchSetting />
                    <LockdownModeSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <CensorshipCircumventionListItem />
                  </SettingsGroup>

                  <SettingsGroup>
                    <CustomDnsSettings />
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
