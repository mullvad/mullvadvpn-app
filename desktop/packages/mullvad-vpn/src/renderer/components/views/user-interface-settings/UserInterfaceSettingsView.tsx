import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import {
  MonochromaticTrayIconSetting,
  NotificationsSetting,
  StartMinimizedSetting,
  UnpinnedWindowSetting,
} from '../../../features/client/components';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
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
import { AnimateMapSetting, LanguageListItem } from './components';

const StyledAnimateMapCellGroup = styled(SettingsGroup)({
  '@media (prefers-reduced-motion: reduce)': {
    display: 'none',
  },
});

export function UserInterfaceSettingsView() {
  const { pop } = useHistory();
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('user-interface-settings-view', 'User interface settings')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('user-interface-settings-view', 'User interface settings')}
                </HeaderTitle>
              </SettingsHeader>

              <SettingsContent>
                <SettingsStack>
                  <SettingsGroup>
                    <NotificationsSetting />
                  </SettingsGroup>
                  <SettingsGroup>
                    <MonochromaticTrayIconSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <LanguageListItem />
                  </SettingsGroup>

                  {(window.env.platform === 'win32' ||
                    (window.env.platform === 'darwin' && window.env.development)) && (
                    <SettingsGroup>
                      <UnpinnedWindowSetting />
                    </SettingsGroup>
                  )}

                  {unpinnedWindow && (
                    <SettingsGroup>
                      <StartMinimizedSetting />
                    </SettingsGroup>
                  )}

                  <StyledAnimateMapCellGroup>
                    <AnimateMapSetting />
                  </StyledAnimateMapCellGroup>
                </SettingsStack>
              </SettingsContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
