import { useCallback } from 'react';

import { strings } from '../../config.json';
import { messages } from '../../shared/gettext';
import { getDownloadUrl } from '../../shared/version';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { Colors } from '../tokens';
import { RedButton } from './AppButton';
import * as Cell from './cell';
import { TitleBig } from './common/text';
import { BackAction } from './KeyboardNavigation';
import {
  ButtonStack,
  Footer,
  LabelStack,
  Layout,
  SettingsContainer,
  SettingsContent,
  SettingsGroup,
  SettingsNavigationScrollbars,
  SettingsStack,
} from './Layout';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
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
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('navigation-bar', 'Settings')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <SettingsNavigationScrollbars fillContainer>
              <SettingsContent>
                <SettingsHeader>
                  <TitleBig>{messages.pgettext('navigation-bar', 'Settings')}</TitleBig>
                </SettingsHeader>

                <SettingsStack>
                  {showSubSettings ? (
                    <>
                      <SettingsGroup>
                        <UserInterfaceSettingsButton />
                        <MultihopButton />
                        <DaitaButton />
                        <VpnSettingsButton />
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
                    <AppVersionButton />
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

function AppVersionButton() {
  const appVersion = useSelector((state) => state.version.current);
  const consistentVersion = useSelector((state) => state.version.consistent);
  const upToDateVersion = useSelector((state) => (state.version.suggestedUpgrade ? false : true));
  const suggestedIsBeta = useSelector((state) => state.version.suggestedIsBeta ?? false);
  const isOffline = useSelector((state) => state.connection.isBlocked);

  const { openUrl } = useAppContext();
  const openDownloadLink = useCallback(
    () => openUrl(getDownloadUrl(suggestedIsBeta)),
    [openUrl, suggestedIsBeta],
  );

  let alertIcon;
  let footer;
  if (!consistentVersion || !upToDateVersion) {
    const inconsistentVersionMessage = messages.pgettext(
      'settings-view',
      'App is out of sync. Please quit and restart.',
    );

    const updateAvailableMessage = messages.pgettext(
      'settings-view',
      'Update available. Install the latest app version to stay up to date.',
    );

    const message = !consistentVersion ? inconsistentVersionMessage : updateAvailableMessage;

    alertIcon = <Cell.UntintedIcon source="icon-alert" width={18} tintColor={Colors.red} />;
    footer = (
      <Cell.CellFooter>
        <Cell.CellFooterText>{message}</Cell.CellFooterText>
      </Cell.CellFooter>
    );
  }

  return (
    <>
      <Cell.CellNavigationButton
        disabled={isOffline}
        onClick={openDownloadLink}
        icon={{
          source: 'icon-extLink',
          'aria-label': messages.pgettext('accessibility', 'Opens externally'),
        }}>
        <LabelStack>
          {alertIcon}
          <Cell.Label>{messages.pgettext('settings-view', 'App version')}</Cell.Label>
        </LabelStack>
        <Cell.SubText>{appVersion}</Cell.SubText>
      </Cell.CellNavigationButton>
      {footer}
    </>
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
    <RedButton onClick={quit}>
      {tunnelState.state === 'disconnected'
        ? messages.gettext('Quit')
        : messages.gettext('Disconnect & quit')}
    </RedButton>
  );
}
