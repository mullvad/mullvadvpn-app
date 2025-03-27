import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { usePop } from '../../../history/hooks';
import { Flex, TitleBig } from '../../../lib/components';
import { spacings } from '../../../lib/foundations';
import { AppNavigationHeader } from '../../';
import { measurements } from '../../common-styles';
import { BackAction } from '../../KeyboardNavigation';
import { SettingsContainer, SettingsNavigationScrollbars } from '../../Layout';
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

export const Title = styled(TitleBig)`
  margin: 0 ${spacings.medium} ${spacings.medium};
`;

export const Footer = styled(Flex)`
  margin: ${spacings.large} ${measurements.horizontalViewMargin} ${measurements.verticalViewMargin};
`;

export function SettingsView() {
  const pop = usePop();

  const showSubSettings = useShowSubSettings();
  const showSplitTunneling = useShowSplitTunneling();
  const showDebug = useShowDebug();

  return (
    <BackAction action={pop}>
      <SettingsContainer>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('navigation-bar', 'Settings')
            }
          />

          <SettingsNavigationScrollbars fillContainer>
            <Title>
              {
                // TRANSLATORS: Main title for settings view
                messages.pgettext('navigation-bar', 'Settings')
              }
            </Title>

            <Flex $flexDirection="column" $gap="medium">
              {showSubSettings ? (
                <Flex $flexDirection="column">
                  <DaitaListItem />
                  <MultihopListItem />
                  <VpnSettingsListItem />
                  <UserInterfaceSettingsListItem />

                  {showSplitTunneling && <SplitTunnelingListItem />}
                </Flex>
              ) : (
                <UserInterfaceSettingsListItem />
              )}

              <ApiAccessMethodsListItem />

              <Flex $flexDirection="column">
                <SupportListItem />
                <AppInfoListItem />
              </Flex>

              {showDebug && <DebugListItem />}
            </Flex>
            <Footer>
              <QuitButton />
            </Footer>
          </SettingsNavigationScrollbars>
        </NavigationContainer>
      </SettingsContainer>
    </BackAction>
  );
}
