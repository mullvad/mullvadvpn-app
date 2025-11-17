import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import {
  AnimateMapSetting,
  MonochromaticTrayIconSetting,
  NotificationsSetting,
  StartMinimizedSetting,
  UnpinnedWindowSetting,
} from '../../../features/client/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { LanguageListItem } from './components';

const AnimateMapContainer = styled.div({
  '@media (prefers-reduced-motion: reduce)': {
    display: 'none',
  },
});

export function UserInterfaceSettingsView() {
  const { pop } = useHistory();
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);

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
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('user-interface-settings-view', 'User interface settings')}
                </HeaderTitle>
              </SettingsHeader>

              <FlexColumn gap="medium">
                <NotificationsSetting />
                <MonochromaticTrayIconSetting />
                <LanguageListItem />

                {(window.env.platform === 'win32' ||
                  (window.env.platform === 'darwin' && window.env.development)) && (
                  <UnpinnedWindowSetting />
                )}

                {unpinnedWindow && <StartMinimizedSetting />}
                <AnimateMapContainer>
                  <AnimateMapSetting />
                </AnimateMapContainer>
              </FlexColumn>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
