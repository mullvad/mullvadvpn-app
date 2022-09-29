import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

const StyledCellIcon = styled(Cell.UntintedIcon)({
  marginRight: '8px',
});

export default function UserInterfaceSettings() {
  const { pop } = useHistory();
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('user-interface-settings-view', 'User interface settings')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('user-interface-settings-view', 'User interface settings')}
                </HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <NotificationsSetting />
                </Cell.Group>
                <Cell.Group>
                  <MonochromaticTrayIconSetting />
                </Cell.Group>

                <Cell.Group>
                  <LanguageButton />
                </Cell.Group>

                {(window.env.platform === 'win32' ||
                  (window.env.platform === 'darwin' && window.env.development)) && (
                  <Cell.Group>
                    <UnpinnedWindowSetting />
                  </Cell.Group>
                )}

                {unpinnedWindow && (
                  <Cell.Group>
                    <StartMinimizedSetting />
                  </Cell.Group>
                )}
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function NotificationsSetting() {
  const enableSystemNotifications = useSelector(
    (state) => state.settings.guiSettings.enableSystemNotifications,
  );
  const { setEnableSystemNotifications } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('user-interface-settings-view', 'Notifications')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={enableSystemNotifications} onChange={setEnableSystemNotifications} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {messages.pgettext(
              'user-interface-settings-view',
              'Enable or disable system notifications. The critical notifications will always be displayed.',
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function MonochromaticTrayIconSetting() {
  const monochromaticIcon = useSelector((state) => state.settings.guiSettings.monochromaticIcon);
  const { setMonochromaticIcon } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('user-interface-settings-view', 'Monochromatic tray icon')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={monochromaticIcon} onChange={setMonochromaticIcon} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {messages.pgettext(
              'user-interface-settings-view',
              'Use a monochromatic tray icon instead of a colored one.',
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function UnpinnedWindowSetting() {
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);
  const { setUnpinnedWindow } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('user-interface-settings-view', 'Unpin app from taskbar')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={unpinnedWindow} onChange={setUnpinnedWindow} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {messages.pgettext(
              'user-interface-settings-view',
              'Enable to move the app around as a free-standing window.',
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function StartMinimizedSetting() {
  const startMinimized = useSelector((state) => state.settings.guiSettings.startMinimized);
  const { setStartMinimized } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('user-interface-settings-view', 'Start minimized')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={startMinimized} onChange={setStartMinimized} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {messages.pgettext(
              'user-interface-settings-view',
              'Show only the tray icon when the app starts.',
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function LanguageButton() {
  const history = useHistory();
  const { getPreferredLocaleDisplayName } = useAppContext();
  const preferredLocale = useSelector((state) => state.settings.guiSettings.preferredLocale);
  const localeDisplayName = getPreferredLocaleDisplayName(preferredLocale);

  const navigate = useCallback(() => history.push(RoutePath.selectLanguage), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <StyledCellIcon width={24} height={24} source="icon-language" />
      <Cell.Label>
        {
          // TRANSLATORS: Navigation button to the 'Language' settings view
          messages.pgettext('user-interface-settings-view', 'Language')
        }
      </Cell.Label>
      <Cell.SubText>{localeDisplayName}</Cell.SubText>
    </Cell.CellNavigationButton>
  );
}
