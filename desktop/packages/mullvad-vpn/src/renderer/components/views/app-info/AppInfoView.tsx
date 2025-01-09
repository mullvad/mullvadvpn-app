import { messages } from '../../../../shared/gettext';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer, SettingsGroup, SettingsStack } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { AppVersionListItem, ChangelogListItem } from './components';

export const AppInfoView = () => {
  const { pop } = useHistory();
  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={messages.pgettext('app-info-view', 'App info')} />

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
