import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { useAppContext } from '../../../context';
import { Image } from '../../../lib/components';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from '../../AriaGroup';
import * as Cell from '../../cell';
import { BackAction } from '../../KeyboardNavigation';
import {
  Layout,
  SettingsContainer,
  SettingsContent,
  SettingsGroup,
  SettingsStack,
} from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { SettingsNavigationListItem } from '../../settings-navigation-list-item';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';

const StyledAnimateMapCellGroup = styled(SettingsGroup)({
  '@media (prefers-reduced-motion: reduce)': {
    display: 'none',
  },
});

export function UserInterfaceSettings() {
  const { pop } = useHistory();
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('user-interface-settings-view', 'User interface settings')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('user-interface-settings-view', 'User interface settings')}
                </HeaderTitle>
              </SettingsHeader>

              <SettingsContent>
                <SettingsStack>
                  <SettingsGroup>
                    <NotificationsSetting />
                  </SettingsGroup>
                  <SettingsGroup>
                    <MonochromaticTrayIconSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <LanguageButton />
                  </SettingsGroup>

                  {(window.env.platform === 'win32' ||
                    (window.env.platform === 'darwin' && window.env.development)) && (
                    <SettingsGroup>
                      <UnpinnedWindowSetting />
                    </SettingsGroup>
                  )}

                  {unpinnedWindow && (
                    <SettingsGroup>
                      <StartMinimizedSetting />
                    </SettingsGroup>
                  )}

                  <StyledAnimateMapCellGroup>
                    <AnimateMapSetting />
                  </StyledAnimateMapCellGroup>
                </SettingsStack>
              </SettingsContent>
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

function AnimateMapSetting() {
  const animateMap = useSelector((state) => state.settings.guiSettings.animateMap);
  const { setAnimateMap } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('user-interface-settings-view', 'Animate map')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={animateMap} onChange={setAnimateMap} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {messages.pgettext('user-interface-settings-view', 'Animate map movements.')}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function LanguageButton() {
  const { getPreferredLocaleDisplayName } = useAppContext();
  const preferredLocale = useSelector((state) => state.settings.guiSettings.preferredLocale);
  const localeDisplayName = getPreferredLocaleDisplayName(preferredLocale);

  return (
    <SettingsNavigationListItem to={RoutePath.selectLanguage}>
      <SettingsNavigationListItem.Group>
        <Image source="icon-language" />
        <SettingsNavigationListItem.Label>
          {
            // TRANSLATORS: Navigation button to the 'Language' settings view
            messages.pgettext('user-interface-settings-view', 'Language')
          }
        </SettingsNavigationListItem.Label>
      </SettingsNavigationListItem.Group>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>{localeDisplayName}</SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}
