import * as React from 'react';
import { messages } from '../../shared/gettext';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
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
  unpinnedWindow: boolean;
  setAutoStart: (autoStart: boolean) => void;
  setEnableSystemNotifications: (flag: boolean) => void;
  setAutoConnect: (autoConnect: boolean) => void;
  setAllowLan: (allowLan: boolean) => void;
  setShowBetaReleases: (showBetaReleases: boolean) => void;
  setStartMinimized: (startMinimized: boolean) => void;
  setMonochromaticIcon: (monochromaticIcon: boolean) => void;
  setUnpinnedWindow: (unpinnedWindow: boolean) => void;
  onClose: () => void;
}

export default class Preferences extends React.Component<IProps> {
  public render() {
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
                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('preferences-view', 'Launch app on start-up')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch isOn={this.props.autoStart} onChange={this.props.setAutoStart} />
                    </AriaInput>
                  </Cell.Container>
                </AriaInputGroup>
                <StyledSeparator />

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('preferences-view', 'Auto-connect')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        isOn={this.props.autoConnect}
                        onChange={this.props.setAutoConnect}
                      />
                    </AriaInput>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Automatically connect to a server when the app launches.',
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('preferences-view', 'Local network sharing')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch isOn={this.props.allowLan} onChange={this.props.setAllowLan} />
                    </AriaInput>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Allows access to other devices on the same network for sharing, printing etc.',
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('preferences-view', 'Notifications')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        isOn={this.props.enableSystemNotifications}
                        onChange={this.props.setEnableSystemNotifications}
                      />
                    </AriaInput>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Enable or disable system notifications. The critical notifications will always be displayed.',
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('preferences-view', 'Monochromatic tray icon')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        isOn={this.props.monochromaticIcon}
                        onChange={this.props.setMonochromaticIcon}
                      />
                    </AriaInput>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Use a monochromatic tray icon instead of a colored one.',
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                {(process.platform === 'win32' ||
                  (process.platform === 'darwin' && process.env.NODE_ENV === 'development')) && (
                  <AriaInputGroup>
                    <Cell.Container>
                      <AriaLabel>
                        <Cell.InputLabel>
                          {messages.pgettext('preferences-view', 'Unpin app from taskbar')}
                        </Cell.InputLabel>
                      </AriaLabel>
                      <AriaInput>
                        <Cell.Switch
                          isOn={this.props.unpinnedWindow}
                          onChange={this.props.setUnpinnedWindow}
                        />
                      </AriaInput>
                    </Cell.Container>
                    <Cell.Footer>
                      <AriaDescription>
                        <Cell.FooterText>
                          {messages.pgettext(
                            'preferences-view',
                            'Enable to move the app around as a free-standing window.',
                          )}
                        </Cell.FooterText>
                      </AriaDescription>
                    </Cell.Footer>
                  </AriaInputGroup>
                )}

                {this.props.unpinnedWindow && (
                  <React.Fragment>
                    <AriaInputGroup>
                      <Cell.Container>
                        <AriaLabel>
                          <Cell.InputLabel>
                            {messages.pgettext('preferences-view', 'Start minimized')}
                          </Cell.InputLabel>
                        </AriaLabel>
                        <AriaInput>
                          <Cell.Switch
                            isOn={this.props.startMinimized}
                            onChange={this.props.setStartMinimized}
                          />
                        </AriaInput>
                      </Cell.Container>
                      <Cell.Footer>
                        <AriaDescription>
                          <Cell.FooterText>
                            {messages.pgettext(
                              'preferences-view',
                              'Show only the tray icon when the app starts.',
                            )}
                          </Cell.FooterText>
                        </AriaDescription>
                      </Cell.Footer>
                    </AriaInputGroup>
                  </React.Fragment>
                )}

                <AriaInputGroup>
                  <Cell.Container disabled={this.props.isBeta}>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('preferences-view', 'Beta program')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        isOn={this.props.showBetaReleases}
                        onChange={this.props.setShowBetaReleases}
                      />
                    </AriaInput>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
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
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </StyledContainer>
      </Layout>
    );
  }
}
