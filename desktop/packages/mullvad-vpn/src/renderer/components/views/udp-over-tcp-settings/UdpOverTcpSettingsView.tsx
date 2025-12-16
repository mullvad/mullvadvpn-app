import { messages } from '../../../../shared/gettext';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
import { UdpOverTcpPortSetting } from './components';

export function UdpOverTcpSettingsView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('wireguard-settings-nav', 'UDP-over-TCP')
            }
          />

          <NavigationScrollbars>
            <View.Content>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'UDP-over-TCP')}
                </HeaderTitle>

                <UdpOverTcpPortSetting />
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
