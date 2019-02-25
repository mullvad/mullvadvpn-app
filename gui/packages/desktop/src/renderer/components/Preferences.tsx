import { HeaderTitle, SettingsHeader } from '@mullvad/components';
import * as React from 'react';
import { Component, View } from 'reactxp';
import { pgettext } from '../../shared/gettext';
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
import Switch from './Switch';

export interface IPreferencesProps {
  autoStart: boolean;
  autoConnect: boolean;
  allowLan: boolean;
  monochromaticIcon: boolean;
  startMinimized: boolean;
  enableMonochromaticIconToggle: boolean;
  enableStartMinimizedToggle: boolean;
  setAutoStart: (autoStart: boolean) => void;
  setAutoConnect: (autoConnect: boolean) => void;
  setAllowLan: (allowLan: boolean) => void;
  setStartMinimized: (startMinimized: boolean) => void;
  setMonochromaticIcon: (monochromaticIcon: boolean) => void;
  onClose: () => void;
}

export default class Preferences extends Component<IPreferencesProps> {
  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.preferences}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose}>
                  {// TRANSLATORS: Settings
                  pgettext('preferences-view', 'back-bar-item')}
                </BackBarItem>
                <TitleBarItem>
                  {// TRANSLATORS: Preferences
                  pgettext('preferences-view', 'title-bar-item')}
                </TitleBarItem>
              </NavigationBar>

              <View style={styles.preferences__container}>
                <NavigationScrollbars>
                  <SettingsHeader>
                    <HeaderTitle>
                      {// TRANSLATORS: Preferences
                      pgettext('preferences-view', 'header-title')}
                    </HeaderTitle>
                  </SettingsHeader>

                  <View style={styles.preferences__content}>
                    <Cell.Container>
                      <Cell.Label>
                        {// TRANSLATORS: Launch app on start-up
                        pgettext('preferences-view', 'auto-start-label')}
                      </Cell.Label>
                      <Switch isOn={this.props.autoStart} onChange={this.onChangeAutoStart} />
                    </Cell.Container>
                    <View style={styles.preferences__separator} />

                    <Cell.Container>
                      <Cell.Label>
                        {// TRANSLATORS: Auto-connect
                        pgettext('preferences-view', 'auto-connect-label')}
                      </Cell.Label>
                      <Switch isOn={this.props.autoConnect} onChange={this.props.setAutoConnect} />
                    </Cell.Container>
                    <Cell.Footer>
                      {// TRANSLATORS: Automatically connect to a server when the app launches.
                      pgettext('preferences-view', 'auto-connect-footer')}
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>
                        {// TRANSLATORS: Local network sharing
                        pgettext('preferences-view', 'local-network-sharing-label')}
                      </Cell.Label>
                      <Switch isOn={this.props.allowLan} onChange={this.props.setAllowLan} />
                    </Cell.Container>
                    <Cell.Footer>
                      {// TRANSLATORS: Allows access to other devices on the same network for sharing, printing etc.
                      pgettext('preferences-view', 'local-network-sharing-footer')}
                    </Cell.Footer>

                    <MonochromaticIconToggle
                      enable={this.props.enableMonochromaticIconToggle}
                      monochromaticIcon={this.props.monochromaticIcon}
                      onChange={this.props.setMonochromaticIcon}
                    />

                    <StartMinimizedToggle
                      enable={this.props.enableStartMinimizedToggle}
                      startMinimized={this.props.startMinimized}
                      onChange={this.props.setStartMinimized}
                    />
                  </View>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  private onChangeAutoStart = (autoStart: boolean) => {
    this.props.setAutoStart(autoStart);
  };
}

interface IMonochromaticIconProps {
  enable: boolean;
  monochromaticIcon: boolean;
  onChange: (value: boolean) => void;
}

class MonochromaticIconToggle extends Component<IMonochromaticIconProps> {
  public render() {
    if (this.props.enable) {
      return (
        <View>
          <Cell.Container>
            <Cell.Label>
              {// TRANSLATORS: Monochromatic tray icon
              pgettext('preferences-view', 'monochromatic-tray-icon-label')}
            </Cell.Label>
            <Switch isOn={this.props.monochromaticIcon} onChange={this.props.onChange} />
          </Cell.Container>
          <Cell.Footer>
            {// TRANSLATORS: Use a monochromatic tray icon instead of a colored one.
            pgettext('preferences-view', 'monochromatic-tray-icon-footer')}
          </Cell.Footer>
        </View>
      );
    } else {
      return null;
    }
  }
}

interface IStartMinimizedProps {
  enable: boolean;
  startMinimized: boolean;
  onChange: (value: boolean) => void;
}

class StartMinimizedToggle extends Component<IStartMinimizedProps> {
  public render() {
    if (this.props.enable) {
      return (
        <View>
          <Cell.Container>
            <Cell.Label>
              {// TRANSLATORS: Start minimized
              pgettext('preferences-view', 'start-minimized-label')}
            </Cell.Label>
            <Switch isOn={this.props.startMinimized} onChange={this.props.onChange} />
          </Cell.Container>
          <Cell.Footer>
            {// TRANSLATORS: Show only the tray icon when the app starts.
            pgettext('preferences-view', 'start-minimized-footer')}
          </Cell.Footer>
        </View>
      );
    } else {
      return null;
    }
  }
}
