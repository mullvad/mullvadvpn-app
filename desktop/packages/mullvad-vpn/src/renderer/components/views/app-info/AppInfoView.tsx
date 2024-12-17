import { messages } from '../../../../shared/gettext';
import { useHistory } from '../../../lib/history';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer, SettingsGroup, SettingsStack } from '../../Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from '../../NavigationBar';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { AppVersionListItem, ChangelogListItem } from './components';

export const AppInfoView = () => {
  const { pop } = useHistory();
  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>{messages.pgettext('app-info-view', 'App info')}</TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('app-info-view', 'App info')}</HeaderTitle>
              </SettingsHeader>
              <SettingsContainer>
                <SettingsStack>
                  <SettingsGroup>
                    <AppVersionListItem />
                  </SettingsGroup>
                  <SettingsGroup>
                    <ChangelogListItem />
                  </SettingsGroup>
                </SettingsStack>
              </SettingsContainer>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
};
