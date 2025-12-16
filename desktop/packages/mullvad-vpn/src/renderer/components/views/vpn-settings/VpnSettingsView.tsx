import { messages } from '../../../../shared/gettext';
import { AutoConnectSetting, AutoStartSetting } from '../../../features/client/components';
import { AllowLanSetting } from '../../../features/lan-sharing/components';
import {
  EnableIpv6Setting,
  LockdownModeSetting,
  QuantumResistantSetting,
} from '../../../features/tunnel/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
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
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('vpn-settings-view', 'VPN settings')
            }
          />

          <NavigationScrollbars>
            <View.Content>
              <View.Container horizontalMargin="medium" gap="medium" flexDirection="column">
                <HeaderTitle>{messages.pgettext('vpn-settings-view', 'VPN settings')}</HeaderTitle>

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
                  <FlexColumn>
                    <KillSwitchSetting />
                    <LockdownModeSetting />
                  </FlexColumn>
                  <AntiCensorshipListItem position="solo" />
                  <QuantumResistantSetting position="solo" />
                  <IpVersionSetting />
                  <MtuSetting />
                  <IpOverrideSettings position="solo" />
                </FlexColumn>
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
