import { messages } from '../../../../shared/gettext';
import { usePop } from '../../../history/hooks';
import { FlexColumn } from '../../../lib/components/flex-column';
import { ListItemGroup } from '../../../lib/components/list-item-group';
import { View } from '../../../lib/components/view';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../keyboard-navigation';
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
              <View.Container horizontalMargin="medium" gap="large" flexDirection="column">
                <FlexColumn gap="medium">
                  {showSubSettings ? (
                    <>
                      <ListItemGroup variant="grouped">
                        <DaitaListItem />
                        <MultihopListItem />
                        <VpnSettingsListItem />
                        <UserInterfaceSettingsListItem />
                      </ListItemGroup>
                      {showSplitTunneling && <SplitTunnelingListItem />}
                    </>
                  ) : (
                    <UserInterfaceSettingsListItem />
                  )}

                  <ApiAccessMethodsListItem />

                  <ListItemGroup variant="grouped">
                    <SupportListItem />
                    <AppInfoListItem />
                  </ListItemGroup>

                  {showDebug && <DebugListItem />}
                </FlexColumn>
                <QuitButton />
              </View.Container>
            </View.Content>
          </SettingsNavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
