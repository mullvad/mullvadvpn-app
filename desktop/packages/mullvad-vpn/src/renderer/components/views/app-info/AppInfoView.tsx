import { messages } from '../../../../shared/gettext';
import { Flex } from '../../../lib/components';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer, SettingsStack } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { ChangelogListItem, UpdateAvailableListItem, VersionListItem } from './components';
import { BetaListItem } from './components/beta-list-item';

export function AppInfoView() {
  const { pop } = useHistory();
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);
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
              <SettingsContainer>
                <SettingsStack>
                  {suggestedUpgrade && <UpdateAvailableListItem />}
                  <Flex $flexDirection="column">
                    <VersionListItem />
                    <ChangelogListItem />
                  </Flex>
                  <BetaListItem />
                </SettingsStack>
              </SettingsContainer>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
