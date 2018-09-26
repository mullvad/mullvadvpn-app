// @flow

import * as React from 'react';
import { Component, View } from 'reactxp';
import * as Cell from './Cell';
import { Layout, Container } from './Layout';
import NavigationBar, { BackBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import Switch from './Switch';
import styles from './PreferencesStyles';

export type PreferencesProps = {
  autoConnect: boolean,
  allowLan: boolean,
  getAutoStart: () => boolean,
  setAutoStart: (boolean) => void,
  setAutoConnect: (boolean) => void,
  setAllowLan: (boolean) => void,
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
            <NavigationBar>
              <BackBarItem action={this.props.onClose} title={'Settings'} />
            </NavigationBar>

            <View style={styles.preferences__container}>
              <SettingsHeader>
                <HeaderTitle>Preferences</HeaderTitle>
              </SettingsHeader>

              <View style={styles.preferences__content}>
                <Cell.Container>
                  <Cell.Label style={styles.preferences__cell_label}>Auto-connect</Cell.Label>
                  <Switch isOn={this.props.autoConnect} onChange={this.props.setAutoConnect} />
                </Cell.Container>
                <Cell.Footer>Automatically connect the VPN when the computer starts.</Cell.Footer>

                <Cell.Container>
                  <Cell.Label style={styles.preferences__cell_label}>Auto-start</Cell.Label>
                  <Switch isOn={this.state.autoStart} onChange={this._onChangeAutoStart} />
                </Cell.Container>
                <Cell.Footer>Automatically open Mullvad VPN at login to the system.</Cell.Footer>

                <Cell.Container>
                  <Cell.Label style={styles.preferences__cell_label}>
                    Local network sharing
                  </Cell.Label>
                  <Switch isOn={this.props.allowLan} onChange={this.props.setAllowLan} />
                </Cell.Container>
                <Cell.Footer>
                  Allows access to other devices on the same network for sharing, printing etc.
                </Cell.Footer>
              </View>
            </View>
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
