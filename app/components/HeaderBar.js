// @flow
import React from 'react';
import { Component, Text, Button, View } from 'reactxp';

import Img from './Img';

import styles from './HeaderBarStyles';
import platformStyles from './HeaderBarPlatformStyles';

export type HeaderBarStyle = 'default' | 'defaultDark' | 'error' | 'success';
export type HeaderBarProps = {
  style: HeaderBarStyle,
  showSettings: boolean,
  onSettings: ?() => void,
};

export default class HeaderBar extends Component<HeaderBarProps> {
  static defaultProps: HeaderBarProps = {
    style: 'default',
    showSettings: false,
    onSettings: null,
  };

  render() {
    const containerClass = [
      styles['headerbar'],
      platformStyles[process.platform],
      styles['style_' + this.props.style],
    ];

    return (
      <View style={containerClass}>
        <View style={styles.container} testName="headerbar__container">
          <Img height={50} width={50} source="logo-icon" />
          <Text style={styles.title}>MULLVAD VPN</Text>
        </View>

        {this.props.showSettings ? (
          <Button
            style={styles.settings}
            onPress={this.props.onSettings}
            testName="headerbar__settings">
            <Img
              height={24}
              width={24}
              source="icon-settings"
              style={[styles.settings_icon, platformStyles.settings_icon]}
              hoverStyle={styles.settings_icon_hover}
            />
          </Button>
        ) : null}
      </View>
    );
  }
}
