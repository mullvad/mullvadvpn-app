import * as React from 'react';
import { Component, View } from 'reactxp';
import { messages } from '../../shared/gettext';
import * as Cell from './Cell';
import { Container, Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import styles from './PreferencesStyles';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

export interface IProps {
  autoStart: boolean;
  autoConnect: boolean;
  allowLan: boolean;
  showBetaReleases?: boolean;
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

export default class Preferences extends Component<IProps> {
  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.preferences}>
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

              <View style={styles.preferences__container}>
                <NavigationScrollbars>
                  <SettingsHeader>
                    <HeaderTitle>
                      {messages.pgettext('preferences-view', 'Preferences')}
                    </HeaderTitle>
                  </SettingsHeader>

                  <View style={styles.preferences__content}>
                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Launch app on start-up')}
                      </Cell.Label>
                      <Cell.Switch isOn={this.props.autoStart} onChange={this.props.setAutoStart} />
                    </Cell.Container>
                    <View style={styles.preferences__separator} />

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Auto-connect')}
                      </Cell.Label>
                      <Cell.Switch
                        isOn={this.props.autoConnect}
                        onChange={this.props.setAutoConnect}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Automatically connect to a server when the app launches.',
                        )}
                      </Cell.FooterText>
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Local network sharing')}
                      </Cell.Label>
                      <Cell.Switch isOn={this.props.allowLan} onChange={this.props.setAllowLan} />
                    </Cell.Container>
                    <Cell.Footer>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Allows access to other devices on the same network for sharing, printing etc.',
                        )}
                      </Cell.FooterText>
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Notifications')}
                      </Cell.Label>
                      <Cell.Switch
                        isOn={this.props.enableSystemNotifications}
                        onChange={this.props.setEnableSystemNotifications}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Enable or disable system notifications. The critical notifications will always be displayed.',
                        )}
                      </Cell.FooterText>
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Monochromatic tray icon')}
                      </Cell.Label>
                      <Cell.Switch
                        isOn={this.props.monochromaticIcon}
                        onChange={this.props.setMonochromaticIcon}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Use a monochromatic tray icon instead of a colored one.',
                        )}
                      </Cell.FooterText>
                    </Cell.Footer>

                    {this.props.enableStartMinimizedToggle ? (
                      <React.Fragment>
                        <Cell.Container>
                          <Cell.Label>
                            {messages.pgettext('preferences-view', 'Start minimized')}
                          </Cell.Label>
                          <Cell.Switch
                            isOn={this.props.startMinimized}
                            onChange={this.props.setStartMinimized}
                          />
                        </Cell.Container>
                        <Cell.Footer>
                          <Cell.FooterText>
                            {messages.pgettext(
                              'preferences-view',
                              'Show only the tray icon when the app starts.',
                            )}
                          </Cell.FooterText>
                        </Cell.Footer>
                      </React.Fragment>
                    ) : undefined}

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Beta program')}
                      </Cell.Label>
                      <Cell.Switch
                        isOn={this.props.showBetaReleases || false}
                        onChange={this.props.setShowBetaReleases}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'preferences-view',
                          'Enable to get notified when new beta versions of the app are released.',
                        )}
                      </Cell.FooterText>
                    </Cell.Footer>
                  </View>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }
}
