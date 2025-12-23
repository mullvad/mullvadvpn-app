import { sprintf } from 'sprintf-js';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
import { PortSetting } from './components';

export function WireguardPortView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <View.Content>
            <AppNavigationHeader
              title={sprintf(messages.pgettext('wireguard-settings-view', '%(wireGuard)s port'), {
                wireGuard: strings.wireguard,
              })}
            />

            <NavigationScrollbars>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>
                  {sprintf(
                    // TRANSLATORS: Page title for WireGuard port view
                    messages.pgettext('wireguard-settings-view', '%(wireGuard)s port'),
                    {
                      wireGuard: strings.wireguard,
                    },
                  )}
                </HeaderTitle>
                <PortSetting />
              </View.Container>
            </NavigationScrollbars>
          </View.Content>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
