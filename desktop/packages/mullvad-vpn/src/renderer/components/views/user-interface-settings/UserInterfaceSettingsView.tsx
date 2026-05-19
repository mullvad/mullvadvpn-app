import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import {
  AnimateMapSetting,
  MonochromaticTrayIconSetting,
  NotificationsSetting,
  StartMinimizedSetting,
  UnpinnedWindowSetting,
} from '../../../features/client/components';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { isPlatform } from '../../../utils';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
import { LanguageListItem } from './components';

const AnimateMapContainer = styled.div({
  '@media (prefers-reduced-motion: reduce)': {
    display: 'none',
  },
});

export function UserInterfaceSettingsView() {
  const { pop } = useHistory();
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);

  const showUnpinnedWindowSetting = isPlatform('win32') || isPlatform('darwin');
  const showStartMinimizedSetting = showUnpinnedWindowSetting && unpinnedWindow;

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('user-interface-settings-view', 'User interface settings')
            }
          />

          <NavigationScrollbars>
            <View.Content>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>
                  {messages.pgettext('user-interface-settings-view', 'User interface settings')}
                </HeaderTitle>

                <NotificationsSetting position="solo" />
                <MonochromaticTrayIconSetting position="solo" />
                <LanguageListItem position="solo" />
                {showUnpinnedWindowSetting && <UnpinnedWindowSetting position="solo" />}
                {showStartMinimizedSetting && <StartMinimizedSetting position="solo" />}
                <AnimateMapContainer>
                  <AnimateMapSetting />
                </AnimateMapContainer>
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
