import * as React from 'react';
import { messages } from '../../shared/gettext';
import { createInputAriaAttributes } from '../lib/accessibility';
import * as Cell from './Cell';
import { Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import { StyledContainer, StyledContent, StyledSeparator } from './PreferencesStyles';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

export interface IProps {
  autoStart: boolean;
  autoConnect: boolean;
  allowLan: boolean;
  showBetaReleases: boolean;
  isBeta: boolean;
  enableSystemNotifications: boolean;
  monochromaticIcon: boolean;
  startMinimized: boolean;
  enableStartMinimizedToggle: boolean;
  setAutoStart: (autoStart: boolean) => void;
  setEnableSystemNotifications: (flag: boolean) => void;
  setAutoConnect: (autoConnect: boolean) => void;
  setAllowLan: (allowLan: boolean) => void;
  setShowBetaReleases: (showBetaReleases: boolean) => void;
  setStartMinimized: (startMinimized: boolean) => void;
  setMonochromaticIcon: (monochromaticIcon: boolean) => void;
  onClose: () => void;
}

export default class Preferences extends React.Component<IProps> {
  public render() {
    const autoStartAria = createInputAriaAttributes();
    const autoConnectAria = createInputAriaAttributes();
    const allowLanAria = createInputAriaAttributes();
    const systemNotificationsAria = createInputAriaAttributes();
    const monochromaticIconAria = createInputAriaAttributes();
    const startMinimizedAria = createInputAriaAttributes();
    const betaProgramAria = createInputAriaAttributes();

    return (
      <Layout>
        <StyledContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <BackBarItem action={this.props.onClose}>
                  {
                    // TRANSLATORS: Back button in navigation bar
                    messages.pgettext('navigation-bar', 'Settings')
                  }
                </BackBarItem>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('preferences-nav', 'Preferences')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('preferences-view', 'Preferences')}</HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Container>
                  <Cell.InputLabel {...autoStartAria.label}>
                    {messages.pgettext('preferences-view', 'Launch app on start-up')}
                  </Cell.InputLabel>
                  <Cell.Switch
                    isOn={this.props.autoStart}
                    onChange={this.props.setAutoStart}
                    {...autoStartAria.input}
                  />
                </Cell.Container>
                <StyledSeparator />

                <Cell.Container>
                  <Cell.InputLabel {...autoConnectAria.label}>
                    {messages.pgettext('preferences-view', 'Auto-connect')}
                  </Cell.InputLabel>
                  <Cell.Switch
                    isOn={this.props.autoConnect}
                    onChange={this.props.setAutoConnect}
                    {...autoConnectAria.input}
                  />
                </Cell.Container>
                <Cell.Footer>
                  <Cell.FooterText {...autoConnectAria.description}>
                    {messages.pgettext(
                      'preferences-view',
                      'Automatically connect to a server when the app launches.',
                    )}
                  </Cell.FooterText>
                </Cell.Footer>

                <Cell.Container>
                  <Cell.InputLabel {...allowLanAria.label}>
                    {messages.pgettext('preferences-view', 'Local network sharing')}
                  </Cell.InputLabel>
                  <Cell.Switch
                    isOn={this.props.allowLan}
                    onChange={this.props.setAllowLan}
                    {...allowLanAria.input}
                  />
                </Cell.Container>
                <Cell.Footer>
                  <Cell.FooterText {...allowLanAria.description}>
                    {messages.pgettext(
                      'preferences-view',
                      'Allows access to other devices on the same network for sharing, printing etc.',
                    )}
                  </Cell.FooterText>
                </Cell.Footer>

                <Cell.Container>
                  <Cell.InputLabel {...systemNotificationsAria.label}>
                    {messages.pgettext('preferences-view', 'Notifications')}
                  </Cell.InputLabel>
                  <Cell.Switch
                    isOn={this.props.enableSystemNotifications}
                    onChange={this.props.setEnableSystemNotifications}
                    {...systemNotificationsAria.input}
                  />
                </Cell.Container>
                <Cell.Footer>
                  <Cell.FooterText {...systemNotificationsAria.description}>
                    {messages.pgettext(
                      'preferences-view',
                      'Enable or disable system notifications. The critical notifications will always be displayed.',
                    )}
                  </Cell.FooterText>
                </Cell.Footer>

                <Cell.Container>
                  <Cell.InputLabel {...monochromaticIconAria.label}>
                    {messages.pgettext('preferences-view', 'Monochromatic tray icon')}
                  </Cell.InputLabel>
                  <Cell.Switch
                    isOn={this.props.monochromaticIcon}
                    onChange={this.props.setMonochromaticIcon}
                    {...monochromaticIconAria.input}
                  />
                </Cell.Container>
                <Cell.Footer>
                  <Cell.FooterText {...monochromaticIconAria.description}>
                    {messages.pgettext(
                      'preferences-view',
                      'Use a monochromatic tray icon instead of a colored one.',
                    )}
                  </Cell.FooterText>
                </Cell.Footer>

                {this.props.enableStartMinimizedToggle ? (
                  <React.Fragment>
                    <Cell.Container>
                      <Cell.InputLabel {...startMinimizedAria.label}>
                        {messages.pgettext('preferences-view', 'Start minimized')}
                      </Cell.InputLabel>
                      <Cell.Switch
                        isOn={this.props.startMinimized}
                        onChange={this.props.setStartMinimized}
                        {...startMinimizedAria.input}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      <Cell.FooterText {...startMinimizedAria.description}>
                        {messages.pgettext(
                          'preferences-view',
                          'Show only the tray icon when the app starts.',
                        )}
                      </Cell.FooterText>
                    </Cell.Footer>
                  </React.Fragment>
                ) : undefined}

                <Cell.Container disabled={this.props.isBeta}>
                  <Cell.InputLabel {...betaProgramAria.label}>
                    {messages.pgettext('preferences-view', 'Beta program')}
                  </Cell.InputLabel>
                  <Cell.Switch
                    isOn={this.props.showBetaReleases}
                    onChange={this.props.setShowBetaReleases}
                    {...betaProgramAria.input}
                  />
                </Cell.Container>
                <Cell.Footer>
                  <Cell.FooterText {...betaProgramAria.description}>
                    {this.props.isBeta
                      ? messages.pgettext(
                          'preferences-view',
                          'This option is unavailable while using a beta version.',
                        )
                      : messages.pgettext(
                          'preferences-view',
                          'Enable to get notified when new beta versions of the app are released.',
                        )}
                  </Cell.FooterText>
                </Cell.Footer>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </StyledContainer>
      </Layout>
    );
  }
}
