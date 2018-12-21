// @flow

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
  autoConnect: boolean,
  allowLan: boolean,
  monochromaticIcon: boolean,
  startMinimized: boolean,
  enableMonochromaticIconToggle: boolean,
  enableStartMinimizedToggle: boolean,
  getAutoStart: () => boolean,
  setAutoStart: (boolean) => void,
  setAutoConnect: (boolean) => void,
  setAllowLan: (boolean) => void,
  setStartMinimized: (boolean) => void,
  setMonochromaticIcon: (boolean) => void,
  onClose: () => void,
};

type State = {
  autoStart: boolean,
};

export default class Preferences extends Component<PreferencesProps, State> {
  state = {
    autoStart: false,
  };

  constructor(props: PreferencesProps) {
    super();
    this.state.autoStart = props.getAutoStart();
  }

  render() {
    return (
      <Layout>
        <Container>
          <View style={styles.preferences}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose} title={'Settings'} />
                <TitleBarItem>Preferences</TitleBarItem>
              </NavigationBar>

              <View style={styles.preferences__container}>
                <NavigationScrollbars>
                  <SettingsHeader>
                    <HeaderTitle>Preferences</HeaderTitle>
                  </SettingsHeader>

                  <View style={styles.preferences__content}>
                    <Cell.Container>
                      <Cell.Label>Auto-connect</Cell.Label>
                      <Switch isOn={this.props.autoConnect} onChange={this.props.setAutoConnect} />
                    </Cell.Container>
                    <Cell.Footer>
                      Automatically connect to the VPN at the earliest moment during computer
                      boot-up.
                    </Cell.Footer>

                    <Cell.Container>
                      <Cell.Label>Auto-launch</Cell.Label>
                      <Switch isOn={this.state.autoStart} onChange={this._onChangeAutoStart} />
                    </Cell.Container>
                    <Cell.Footer>
                      Automatically launch the app when logging in to the computer.
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
    // TODO: Handle failure to set auto-start
    this.setState({ autoStart });
  };
}

type MonochromaticIconProps = {
  enable: boolean,
  monochromaticIcon: boolean,
  onChange: (boolean) => void,
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
  enable: boolean,
  startMinimized: boolean,
  onChange: (boolean) => void,
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
