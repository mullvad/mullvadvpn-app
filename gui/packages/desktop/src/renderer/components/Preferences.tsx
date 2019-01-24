import * as React from 'react';
import { Component, View } from 'reactxp';
import { SettingsHeader, HeaderTitle } from '@mullvad/components';
import * as Cell from './Cell';
import { Layout, Container } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  BackBarItem,
  TitleBarItem,
} from './NavigationBar';
import Switch from './Switch';
import styles from './PreferencesStyles';

export type PreferencesProps = {
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
};

export default class Preferences extends Component<PreferencesProps> {
  render() {
    return (
      <Layout>
        <Container>
          <View style={styles.preferences}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose}>Settings</BackBarItem>
                <TitleBarItem>Preferences</TitleBarItem>
              </NavigationBar>

              <View style={styles.preferences__container}>
                <NavigationScrollbars>
                  <SettingsHeader>
                    <HeaderTitle>Preferences</HeaderTitle>
                  </SettingsHeader>

                  <View style={styles.preferences__content}>
                    <Cell.Container>
                      <Cell.Label>Launch app on start-up</Cell.Label>
                      <Switch isOn={this.props.autoStart} onChange={this._onChangeAutoStart} />
                    </Cell.Container>
                    <View style={styles.preferences__separator} />

                    <Cell.Container>
                      <Cell.Label>Auto-connect</Cell.Label>
                      <Switch isOn={this.props.autoConnect} onChange={this.props.setAutoConnect} />
                    </Cell.Container>
                    <Cell.Footer>
                      Automatically connect to a server when the app launches.
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>Local network sharing</Cell.Label>
                      <Switch isOn={this.props.allowLan} onChange={this.props.setAllowLan} />
                    </Cell.Container>
                    <Cell.Footer>
                      Allows access to other devices on the same network for sharing, printing etc.
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

  _onChangeAutoStart = (autoStart: boolean) => {
    this.props.setAutoStart(autoStart);
  };
}

type MonochromaticIconProps = {
  enable: boolean;
  monochromaticIcon: boolean;
  onChange: (value: boolean) => void;
};

class MonochromaticIconToggle extends Component<MonochromaticIconProps> {
  render() {
    if (this.props.enable) {
      return (
        <View>
          <Cell.Container>
            <Cell.Label>Monochromatic tray icon</Cell.Label>
            <Switch isOn={this.props.monochromaticIcon} onChange={this.props.onChange} />
          </Cell.Container>
          <Cell.Footer>Use a monochromatic tray icon instead of a colored one.</Cell.Footer>
        </View>
      );
    } else {
      return null;
    }
  }
}

type StartMinimizedProps = {
  enable: boolean;
  startMinimized: boolean;
  onChange: (value: boolean) => void;
};

class StartMinimizedToggle extends Component<StartMinimizedProps> {
  render() {
    if (this.props.enable) {
      return (
        <View>
          <Cell.Container>
            <Cell.Label>Start minimized</Cell.Label>
            <Switch isOn={this.props.startMinimized} onChange={this.props.onChange} />
          </Cell.Container>
          <Cell.Footer>Show only the tray icon when the app starts.</Cell.Footer>
        </View>
      );
    } else {
      return null;
    }
  }
}
