import { messages } from '../../../../shared/gettext';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
import { ShadowsocksPortSetting } from './components';

export function ShadowsocksSettingsView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <View.Content>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('wireguard-settings-nav', 'Shadowsocks')
              }
            />

            <NavigationScrollbars>
              <View.Container horizontalMargin="medium" flexDirection="column">
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'Shadowsocks')}
                </HeaderTitle>
                <ShadowsocksPortSetting />
              </View.Container>
            </NavigationScrollbars>
          </View.Content>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
