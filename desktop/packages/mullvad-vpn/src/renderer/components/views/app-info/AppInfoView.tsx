import { messages } from '../../../../shared/gettext';
import { BetaSetting } from '../../../features/version/components';
import { Flex } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { ChangelogListItem, UpdateAvailableListItem, VersionListItem } from './components';
import { useShowUpdateAvailable } from './hooks';

export function AppInfoView() {
  const { pop } = useHistory();
  const showUpdateAvailable = useShowUpdateAvailable();
  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title of the app info view.
              messages.pgettext('app-info-view', 'App info')
            }
          />

          <NavigationScrollbars>
            <View.Content>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('app-info-view', 'App info')}</HeaderTitle>
              </SettingsHeader>

              <Flex flexDirection="column" gap="medium">
                {showUpdateAvailable && <UpdateAvailableListItem />}
                <Flex flexDirection="column">
                  <VersionListItem />
                  <ChangelogListItem />
                </Flex>
                <BetaSetting />
              </Flex>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
