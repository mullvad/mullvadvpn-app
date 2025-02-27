import { useCallback } from 'react';

import { strings } from '../../shared/constants';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { Button, TitleBig } from '../lib/components';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import * as Cell from './cell';
import { BackAction } from './KeyboardNavigation';
import {
  ButtonStack,
  Footer,
  Layout,
  SettingsContainer,
  SettingsContent,
  SettingsGroup,
  SettingsNavigationScrollbars,
  SettingsStack,
} from './Layout';
import { NavigationContainer } from './NavigationContainer';
import SettingsHeader from './SettingsHeader';

export default function Support() {
  const history = useHistory();

  const loginState = useSelector((state) => state.account.status);
  const connectedToDaemon = useSelector((state) => state.userInterface.connectedToDaemon);
  const isMacOs13OrNewer = useSelector((state) => state.userInterface.isMacOs13OrNewer);

  const showSubSettings = loginState.type === 'ok' && connectedToDaemon;
  const showSplitTunneling = window.env.platform !== 'darwin' || isMacOs13OrNewer;

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('navigation-bar', 'Settings')
              }
            />

            <SettingsNavigationScrollbars fillContainer>
              <SettingsContent>
                <SettingsHeader>
                  <TitleBig>{messages.pgettext('navigation-bar', 'Settings')}</TitleBig>
                </SettingsHeader>

                <SettingsStack>
                  {showSubSettings ? (
                    <>
                      <SettingsGroup>
                        <DaitaButton />
                        <MultihopButton />
                        <VpnSettingsButton />
                        <UserInterfaceSettingsButton />
                      </SettingsGroup>

                      {showSplitTunneling && (
                        <SettingsGroup>
                          <SplitTunnelingButton />
                        </SettingsGroup>
                      )}
                    </>
                  ) : (
                    <SettingsGroup>
                      <UserInterfaceSettingsButton />
                    </SettingsGroup>
                  )}

                  <SettingsGroup>
                    <ApiAccessMethodsButton />
                  </SettingsGroup>

                  <SettingsGroup>
                    <SupportButton />
                    <AppInfoButton />
                  </SettingsGroup>

                  {window.env.development && (
                    <SettingsGroup>
                      <DebugButton />
                    </SettingsGroup>
                  )}
                </SettingsStack>
                <Footer>
                  <ButtonStack>
                    <QuitButton />
                  </ButtonStack>
                </Footer>
              </SettingsContent>
            </SettingsNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function UserInterfaceSettingsButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.userInterfaceSettings), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>
        {
          // TRANSLATORS: Navigation button to the 'User interface settings' view
          messages.pgettext('settings-view', 'User interface settings')
        }
      </Cell.Label>
    </Cell.CellNavigationButton>
  );
}

function MultihopButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.multihopSettings), [history]);
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>{messages.pgettext('settings-view', 'Multihop')}</Cell.Label>
      <Cell.SubText>
        {multihop && !unavailable ? messages.gettext('On') : messages.gettext('Off')}
      </Cell.SubText>
    </Cell.CellNavigationButton>
  );
}

function DaitaButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.daitaSettings), [history]);
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>{strings.daita}</Cell.Label>
      <Cell.SubText>
        {daita && !unavailable ? messages.gettext('On') : messages.gettext('Off')}
      </Cell.SubText>
    </Cell.CellNavigationButton>
  );
}

function VpnSettingsButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.vpnSettings), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>
        {
          // TRANSLATORS: Navigation button to the 'VPN settings' view
          messages.pgettext('settings-view', 'VPN settings')
        }
      </Cell.Label>
    </Cell.CellNavigationButton>
  );
}

function SplitTunnelingButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.splitTunneling), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>{strings.splitTunneling}</Cell.Label>
    </Cell.CellNavigationButton>
  );
}

function ApiAccessMethodsButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.apiAccessMethods), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>
        {
          // TRANSLATORS: Navigation button to the 'API access methods' view
          messages.pgettext('settings-view', 'API access')
        }
      </Cell.Label>
    </Cell.CellNavigationButton>
  );
}

function AppInfoButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.appInfo), [history]);
  const appVersion = useSelector((state) => state.version.current);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>{messages.pgettext('settings-view', 'App info')}</Cell.Label>
      <Cell.SubText>{appVersion}</Cell.SubText>
    </Cell.CellNavigationButton>
  );
}

function SupportButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.support), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>{messages.pgettext('settings-view', 'Support')}</Cell.Label>
    </Cell.CellNavigationButton>
  );
}

function DebugButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.debug), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>Developer tools</Cell.Label>
    </Cell.CellNavigationButton>
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
