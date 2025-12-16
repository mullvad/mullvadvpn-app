import { messages } from '../../../../shared/gettext';
import { BetaSetting } from '../../../features/version/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
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
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>{messages.pgettext('app-info-view', 'App info')}</HeaderTitle>

                <FlexColumn gap="medium">
                  {showUpdateAvailable && <UpdateAvailableListItem />}
                  <FlexColumn>
                    <ChangelogListItem />
                    <VersionListItem />
                  </FlexColumn>
                  <BetaSetting />
                </FlexColumn>
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
