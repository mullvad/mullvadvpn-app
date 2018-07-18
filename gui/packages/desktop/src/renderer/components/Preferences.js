// @flow

import * as React from 'react';
import { Component, Text, View } from 'reactxp';
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
                <View style={styles.preferences__cell}>
                  <View style={styles.preferences__cell_label_container}>
                    <Text style={styles.preferences__cell_label}>Auto-connect</Text>
                  </View>
                  <View style={styles.preferences__cell_accessory}>
                    <Switch isOn={this.props.autoConnect} onChange={this.props.setAutoConnect} />
                  </View>
                </View>
                <View style={styles.preferences__cell_footer}>
                  <Text style={styles.preferences__cell_footer_label}>
                    {'Automatically connect the VPN when the computer starts.'}
                  </Text>
                </View>

                <View style={styles.preferences__cell}>
                  <View style={styles.preferences__cell_label_container}>
                    <Text style={styles.preferences__cell_label}>Auto-start</Text>
                  </View>
                  <View style={styles.preferences__cell_accessory}>
                    <Switch isOn={this.state.autoStart} onChange={this._onChangeAutoStart} />
                  </View>
                </View>
                <View style={styles.preferences__cell_footer}>
                  <Text style={styles.preferences__cell_footer_label}>
                    {'Automatically open Mullvad VPN at login to the system.'}
                  </Text>
                </View>

                <View style={styles.preferences__cell}>
                  <View style={styles.preferences__cell_label_container}>
                    <Text style={styles.preferences__cell_label}>Local network sharing</Text>
                  </View>
                  <View style={styles.preferences__cell_accessory}>
                    <Switch isOn={this.props.allowLan} onChange={this.props.setAllowLan} />
                  </View>
                </View>
                <View style={styles.preferences__cell_footer}>
                  <Text style={styles.preferences__cell_footer_label}>
                    {
                      'Allows access to other devices on the same network for sharing, printing etc.'
                    }
                  </Text>
                </View>
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
