import { messages } from '../../../../shared/gettext';
import { BetaSetting } from '../../../features/version/components';
import { Flex } from '../../../lib/components';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { ChangelogListItem, UpdateAvailableListItem, VersionListItem } from './components';
import { useShowUpdateAvailable } from './hooks';

export function AppInfoView() {
  const { pop } = useHistory();
  const showUpdateAvailable = useShowUpdateAvailable();
  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title of the app info view.
                messages.pgettext('app-info-view', 'App info')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('app-info-view', 'App info')}</HeaderTitle>
              </SettingsHeader>

              <Flex $flexDirection="column" $gap="medium">
                {showUpdateAvailable && <UpdateAvailableListItem />}
                <Flex $flexDirection="column">
                  <VersionListItem />
                  <ChangelogListItem />
                </Flex>
                <BetaSetting />
              </Flex>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
