import { messages } from '../../../../shared/gettext';
import { usePop } from '../../../history/hooks';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { SettingsNavigationScrollbars } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import {
  ApiAccessMethodsListItem,
  AppInfoListItem,
  DaitaListItem,
  DebugListItem,
  MultihopListItem,
  QuitButton,
  SplitTunnelingListItem,
  SupportListItem,
  UserInterfaceSettingsListItem,
  VpnSettingsListItem,
} from './components';
import { useShowDebug, useShowSplitTunneling, useShowSubSettings } from './hooks';

export function SettingsView() {
  const pop = usePop();

  const showSubSettings = useShowSubSettings();
  const showSplitTunneling = useShowSplitTunneling();
  const showDebug = useShowDebug();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('settings-view', 'Settings')
            }
            titleVisible
          />

          <SettingsNavigationScrollbars fillContainer>
            <View.Content>
              <FlexColumn gap="large">
                <FlexColumn gap="medium">
                  {showSubSettings ? (
                    <>
                      <FlexColumn>
                        <DaitaListItem />
                        <MultihopListItem />
                        <VpnSettingsListItem />
                        <UserInterfaceSettingsListItem />
                      </FlexColumn>
                      {showSplitTunneling && <SplitTunnelingListItem />}
                    </>
                  ) : (
                    <UserInterfaceSettingsListItem />
                  )}

                  <ApiAccessMethodsListItem />

                  <FlexColumn>
                    <SupportListItem />
                    <AppInfoListItem />
                  </FlexColumn>

                  {showDebug && <DebugListItem />}
                </FlexColumn>
                <View.Container indent="medium">
                  <QuitButton />
                </View.Container>
              </FlexColumn>
            </View.Content>
          </SettingsNavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
