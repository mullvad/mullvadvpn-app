import * as React from 'react';
import { Component, View } from 'reactxp';
import { messages } from '../../shared/gettext';
import * as Cell from './Cell';
import { Container, Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import styles from './PreferencesStyles';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import Switch from './Switch';

export interface IProps {
  autoStart: boolean;
  autoConnect: boolean;
  allowLan: boolean;
  enableSystemNotifications: boolean;
  monochromaticIcon: boolean;
  startMinimized: boolean;
  enableMonochromaticIconToggle: boolean;
  enableStartMinimizedToggle: boolean;
  setAutoStart: (autoStart: boolean) => void;
  setEnableSystemNotifications: (flag: boolean) => void;
  setAutoConnect: (autoConnect: boolean) => void;
  setAllowLan: (allowLan: boolean) => void;
  setStartMinimized: (startMinimized: boolean) => void;
  setMonochromaticIcon: (monochromaticIcon: boolean) => void;
  onClose: () => void;
}

export default class Preferences extends Component<IProps> {
  private autoStartSwitch = React.createRef<Switch>();
  private autoConnectSwitch = React.createRef<Switch>();
  private allowLanSwitch = React.createRef<Switch>();
  private notificationSwitch = React.createRef<Switch>();
  private monoIconSwitch = React.createRef<Switch>();
  private startMinimizedSwitch = React.createRef<Switch>();

  public componentDidUpdate(prevProps: IProps) {
    if (prevProps.autoStart !== this.props.autoStart && this.autoStartSwitch.current) {
      this.autoStartSwitch.current.setOn(this.props.autoStart);
    }

    if (prevProps.autoConnect !== this.props.autoConnect && this.autoConnectSwitch.current) {
      this.autoConnectSwitch.current.setOn(this.props.autoConnect);
    }

    if (prevProps.allowLan !== this.props.allowLan && this.allowLanSwitch.current) {
      this.allowLanSwitch.current.setOn(this.props.allowLan);
    }

    if (
      prevProps.enableSystemNotifications !== this.props.enableSystemNotifications &&
      this.notificationSwitch.current
    ) {
      this.notificationSwitch.current.setOn(this.props.enableSystemNotifications);
    }

    if (
      prevProps.monochromaticIcon !== this.props.monochromaticIcon &&
      this.monoIconSwitch.current
    ) {
      this.monoIconSwitch.current.setOn(this.props.monochromaticIcon);
    }

    if (
      prevProps.startMinimized !== this.props.startMinimized &&
      this.startMinimizedSwitch.current
    ) {
      this.startMinimizedSwitch.current.setOn(this.props.startMinimized);
    }
  }

  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.preferences}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose}>
                  {// TRANSLATORS: Back button in navigation bar
                  messages.pgettext('preferences-nav', 'Settings')}
                </BackBarItem>
                <TitleBarItem>
                  {// TRANSLATORS: Title label in navigation bar
                  messages.pgettext('preferences-nav', 'Preferences')}
                </TitleBarItem>
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
                      <Cell.Switch
                        defaultOn={this.props.autoStart}
                        onChange={this.props.setAutoStart}
                      />
                    </Cell.Container>
                    <View style={styles.preferences__separator} />

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Auto-connect')}
                      </Cell.Label>
                      <Cell.Switch
                        ref={this.autoConnectSwitch}
                        defaultOn={this.props.autoConnect}
                        onChange={this.props.setAutoConnect}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      {messages.pgettext(
                        'preferences-view',
                        'Automatically connect to a server when the app launches.',
                      )}
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Local network sharing')}
                      </Cell.Label>
                      <Cell.Switch
                        ref={this.allowLanSwitch}
                        defaultOn={this.props.allowLan}
                        onChange={this.props.setAllowLan}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      {messages.pgettext(
                        'preferences-view',
                        'Allows access to other devices on the same network for sharing, printing etc.',
                      )}
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>
                        {messages.pgettext('preferences-view', 'Notifications')}
                      </Cell.Label>
                      <Cell.Switch
                        ref={this.notificationSwitch}
                        defaultOn={this.props.enableSystemNotifications}
                        onChange={this.props.setEnableSystemNotifications}
                      />
                    </Cell.Container>
                    <Cell.Footer>
                      {messages.pgettext(
                        'preferences-view',
                        'Enable or disable system notifications. The critical notifications will always be displayed.',
                      )}
                    </Cell.Footer>

                    {this.props.enableMonochromaticIconToggle ? (
                      <React.Fragment>
                        <Cell.Container>
                          <Cell.Label>
                            {messages.pgettext('preferences-view', 'Monochromatic tray icon')}
                          </Cell.Label>
                          <Cell.Switch
                            ref={this.monoIconSwitch}
                            defaultOn={this.props.monochromaticIcon}
                            onChange={this.props.setMonochromaticIcon}
                          />
                        </Cell.Container>
                        <Cell.Footer>
                          {messages.pgettext(
                            'preferences-view',
                            'Use a monochromatic tray icon instead of a colored one.',
                          )}
                        </Cell.Footer>
                      </React.Fragment>
                    ) : (
                      undefined
                    )}

                    {this.props.enableStartMinimizedToggle ? (
                      <React.Fragment>
                        <Cell.Container>
                          <Cell.Label>
                            {messages.pgettext('preferences-view', 'Start minimized')}
                          </Cell.Label>
                          <Cell.Switch
                            ref={this.startMinimizedSwitch}
                            defaultOn={this.props.startMinimized}
                            onChange={this.props.setStartMinimized}
                          />
                        </Cell.Container>
                        <Cell.Footer>
                          {messages.pgettext(
                            'preferences-view',
                            'Show only the tray icon when the app starts.',
                          )}
                        </Cell.Footer>
                      </React.Fragment>
                    ) : (
                      undefined
                    )}
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
