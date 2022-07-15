import { useCallback, useEffect } from 'react';

import { colors, links } from '../../config.json';
import { formatRemainingTime, hasExpired } from '../../shared/account-expiry';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import * as Cell from './cell';
import { BackAction } from './KeyboardNavigation';
import { Layout } from './Layout';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import {
  StyledCellIcon,
  StyledContainer,
  StyledContent,
  StyledNavigationScrollbars,
  StyledOutOfTimeSubText,
  StyledQuitButton,
  StyledSettingsContent,
} from './SettingsStyles';

export default function Support() {
  const history = useHistory();
  const { updateAccountData } = useAppContext();

  useEffect(() => {
    if (history.action === 'PUSH') {
      updateAccountData();
    }
  }, []);

  const loginState = useSelector((state) => state.account.status);
  const connectedToDaemon = useSelector((state) => state.userInterface.connectedToDaemon);

  const showLargeTitle = loginState.type !== 'ok';
  const showSubSettings = loginState.type === 'ok' && connectedToDaemon;

  return (
    <BackAction icon="close" action={history.dismiss}>
      <Layout>
        <StyledContainer>
          <NavigationContainer>
            <NavigationBar alwaysDisplayBarTitle={!showLargeTitle}>
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
                {showLargeTitle && (
                  <SettingsHeader>
                    <HeaderTitle>{messages.pgettext('navigation-bar', 'Settings')}</HeaderTitle>
                  </SettingsHeader>
                )}

                <StyledSettingsContent>
                  {showSubSettings && (
                    <>
                      <Cell.Group>
                        <AccountButton />
                        <InterfaceSettingsButton />
                        <VpnSettingsButton />
                      </Cell.Group>

                      {(window.env.platform === 'linux' || window.env.platform === 'win32') && (
                        <Cell.Group>
                          <SplitTunnelingButton />
                        </Cell.Group>
                      )}
                    </>
                  )}

                  <Cell.Group>
                    <SupportButton />
                    <AppVersionButton />
                  </Cell.Group>
                </StyledSettingsContent>
              </StyledContent>

              <QuitButton />
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </StyledContainer>
      </Layout>
    </BackAction>
  );
}

function AccountButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.accountSettings), [history]);

  const accountExpiry = useSelector((state) => state.account.expiry);
  const isOutOfTime = accountExpiry ? hasExpired(accountExpiry) : false;
  const formattedExpiry = accountExpiry ? formatRemainingTime(accountExpiry).toUpperCase() : '';
  const outOfTimeMessage = messages.pgettext('settings-view', 'OUT OF TIME');

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>
        {
          // TRANSLATORS: Navigation button to the 'Account' view
          messages.pgettext('settings-view', 'Account')
        }
      </Cell.Label>
      <StyledOutOfTimeSubText isOutOfTime={isOutOfTime}>
        {isOutOfTime ? outOfTimeMessage : formattedExpiry}
      </StyledOutOfTimeSubText>
    </Cell.CellNavigationButton>
  );
}

function InterfaceSettingsButton() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.interfaceSettings), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>
        {
          // TRANSLATORS: Navigation button to the 'Interface settings' view
          messages.pgettext('settings-view', 'Interface settings')
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
      <Cell.Label>
        {
          // TRANSLATORS: Navigation button to the 'Split tunneling' view
          messages.pgettext('settings-view', 'Split tunneling')
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
    () => openUrl(suggestedIsBeta ? links.betaDownload : links.download),
    [openUrl, suggestedIsBeta],
  );

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
      <Cell.Footer>
        <Cell.FooterText>{message}</Cell.FooterText>
      </Cell.Footer>
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

function QuitButton() {
  const { quit } = useAppContext();

  return (
    <StyledQuitButton onClick={quit}>
      {messages.pgettext('settings-view', 'Quit app')}
    </StyledQuitButton>
  );
}
