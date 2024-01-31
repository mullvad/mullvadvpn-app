import { useCallback } from 'react';

import { colors, strings } from '../../config.json';
import { messages } from '../../shared/gettext';
import { getDownloadUrl } from '../../shared/version';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import * as Cell from './cell';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import {
  StyledCellIcon,
  StyledContent,
  StyledNavigationScrollbars,
  StyledQuitButton,
  StyledSettingsContent,
} from './SettingsStyles';

export default function Support() {
  const history = useHistory();

  const loginState = useSelector((state) => state.account.status);
  const connectedToDaemon = useSelector((state) => state.userInterface.connectedToDaemon);

  const showSubSettings = loginState.type === 'ok' && connectedToDaemon;

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

            <StyledNavigationScrollbars fillContainer>
              <StyledContent>
                <SettingsHeader>
                  <HeaderTitle>{messages.pgettext('navigation-bar', 'Settings')}</HeaderTitle>
                </SettingsHeader>

                <StyledSettingsContent>
                  {showSubSettings ? (
                    <>
                      <Cell.Group>
                        <UserInterfaceSettingsButton />
                        <VpnSettingsButton />
                      </Cell.Group>

                      {(window.env.platform === 'linux' || window.env.platform === 'win32') && (
                        <Cell.Group>
                          <SplitTunnelingButton />
                        </Cell.Group>
                      )}
                    </>
                  ) : (
                    <Cell.Group>
                      <UserInterfaceSettingsButton />
                    </Cell.Group>
                  )}

                  <Cell.Group>
                    <ApiAccessMethodsButton />
                  </Cell.Group>

                  <Cell.Group>
                    <SupportButton />
                    <AppVersionButton />
                  </Cell.Group>

                  {window.env.development && (
                    <Cell.Group>
                      <DebugButton />
                    </Cell.Group>
                  )}
                </StyledSettingsContent>
              </StyledContent>

              <QuitButton />
            </StyledNavigationScrollbars>
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
  const openDownloadLink = useCallback(() => openUrl(getDownloadUrl(suggestedIsBeta)), [
    openUrl,
    suggestedIsBeta,
  ]);

  let icon;
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

    icon = <StyledCellIcon source="icon-alert" width={18} tintColor={colors.red} />;
    footer = (
      <Cell.CellFooter>
        <Cell.CellFooterText>{message}</Cell.CellFooterText>
      </Cell.CellFooter>
    );
  }

  return (
    <AriaDescriptionGroup>
      <AriaDescribed>
        <Cell.CellButton disabled={isOffline} onClick={openDownloadLink}>
          {icon}
          <Cell.Label>{messages.pgettext('settings-view', 'App version')}</Cell.Label>
          <Cell.SubText>{appVersion}</Cell.SubText>
          <AriaDescription>
            <Cell.Icon
              height={16}
              width={16}
              source="icon-extLink"
              aria-label={messages.pgettext('accessibility', 'Opens externally')}
            />
          </AriaDescription>
        </Cell.CellButton>
      </AriaDescribed>
      {footer}
    </AriaDescriptionGroup>
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
    <StyledQuitButton onClick={quit}>
      {tunnelState.state === 'disconnected'
        ? messages.gettext('Quit')
        : messages.gettext('Disconnect & quit')}
    </StyledQuitButton>
  );
}
