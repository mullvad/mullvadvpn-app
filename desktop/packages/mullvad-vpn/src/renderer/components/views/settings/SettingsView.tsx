import styled from 'styled-components';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Button, Flex, Icon, TitleBig } from '../../../lib/components';
import { Dot } from '../../../lib/components/dot';
import { ListItem } from '../../../lib/components/list-item';
import { spacings } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { RoutePath } from '../../../lib/routes';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../../';
import { measurements } from '../../common-styles';
import { BackAction } from '../../KeyboardNavigation';
import { SettingsContainer, SettingsNavigationScrollbars } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationListItem } from '../../NavigationListItem';

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

              <ApiAccessMethodsButton />

              <Flex $flexDirection="column">
                <SupportListItem />
                <AppInfoListItem />
              </Flex>

              {window.env.development && <DebugButton />}
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

function UserInterfaceSettingsListItem() {
  return (
    <NavigationListItem to={RoutePath.userInterfaceSettings}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'User interface settings' view
          messages.pgettext('settings-view', 'User interface settings')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

function MultihopListItem() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  return (
    <NavigationListItem to={RoutePath.multihopSettings}>
      <ListItem.Label>{messages.pgettext('settings-view', 'Multihop')}</ListItem.Label>
      <ListItem.Group>
        <ListItem.Text>
          {multihop && !unavailable ? messages.gettext('On') : messages.gettext('Off')}
        </ListItem.Text>
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}

function DaitaListItem() {
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  return (
    <NavigationListItem to={RoutePath.daitaSettings}>
      <ListItem.Label>{strings.daita}</ListItem.Label>
      <ListItem.Group>
        <ListItem.Text>
          {daita && !unavailable ? messages.gettext('On') : messages.gettext('Off')}
        </ListItem.Text>
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}

function VpnSettingsListItem() {
  return (
    <NavigationListItem to={RoutePath.vpnSettings}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'VPN settings' view
          messages.pgettext('settings-view', 'VPN settings')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

function SplitTunnelingListItem() {
  return (
    <NavigationListItem to={RoutePath.splitTunneling}>
      <ListItem.Label>{strings.splitTunneling}</ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

function ApiAccessMethodsButton() {
  return (
    <NavigationListItem to={RoutePath.apiAccessMethods}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'API access methods' view
          messages.pgettext('settings-view', 'API access')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

function AppInfoListItem() {
  const appVersion = useSelector((state) => state.version.current);
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);

  return (
    <NavigationListItem to={RoutePath.appInfo}>
      <Flex $flexDirection="column">
        <ListItem.Label>
          {
            // TRANSLATORS: Navigation button to the 'App info' view
            messages.pgettext('settings-view', 'App info')
          }
        </ListItem.Label>
        {suggestedUpgrade && (
          <StyledText variant="footnoteMini">
            {
              // TRANSLATORS: Label for the app info list item indicating that an update is available and can be downloaded
              messages.pgettext('settings-view', 'Update available')
            }
          </StyledText>
        )}
      </Flex>
      <ListItem.Group>
        <ListItem.Text>{appVersion}</ListItem.Text>
        {suggestedUpgrade && <Dot variant="warning" size="small" />}
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}

function SupportListItem() {
  return (
    <NavigationListItem to={RoutePath.support}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'Support' view
          messages.pgettext('settings-view', 'Support')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

function DebugButton() {
  return (
    <NavigationListItem to={RoutePath.debug}>
      <ListItem.Label>Developer tools</ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

function QuitButton() {
  const { quit } = useAppContext();
  const tunnelState = useSelector((state) => state.connection.status);

  return (
    <Button variant="destructive" onClick={quit}>
      <Button.Text>
        {tunnelState.state === 'disconnected'
          ? messages.gettext('Quit')
          : messages.gettext('Disconnect & quit')}
      </Button.Text>
    </Button>
  );
}
