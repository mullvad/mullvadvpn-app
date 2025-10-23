import { messages } from '../../../../shared/gettext';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { SettingsContent } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { PortSetting } from './components';

export function WireguardPortView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={messages.pgettext('wireguard-settings-view', 'WireGuard port')}
          />

          <NavigationScrollbars>
            <SettingsHeader>
              <HeaderTitle>
                {
                  // TRANSLATORS: Page title for WireGuard port view
                  messages.pgettext('wireguard-settings-view', 'WireGuard port')
                }
              </HeaderTitle>
            </SettingsHeader>

            <SettingsContent>
              <PortSetting />
            </SettingsContent>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
