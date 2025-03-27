import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { Flex, TitleBig } from '../../../lib/components';
import { spacings } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
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

export const Title = styled(TitleBig)`
  margin: 0 ${spacings.medium} ${spacings.medium};
`;

export const Footer = styled(Flex)`
  margin: ${spacings.large} ${measurements.horizontalViewMargin} ${measurements.verticalViewMargin};
`;

export function SettingsView() {
  const history = useHistory();

  const loginState = useSelector((state) => state.account.status);
  const connectedToDaemon = useSelector((state) => state.userInterface.connectedToDaemon);
  const isMacOs13OrNewer = useSelector((state) => state.userInterface.isMacOs13OrNewer);

  const showSubSettings = loginState.type === 'ok' && connectedToDaemon;
  const showSplitTunneling = window.env.platform !== 'darwin' || isMacOs13OrNewer;

  return (
    <BackAction action={history.pop}>
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

              {window.env.development && <DebugListItem />}
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
