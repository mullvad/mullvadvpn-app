// @flow
import React from 'react';
import {
  Component,
  Text,
  Button,
  View
} from 'reactxp';

import Img from './Img';

import styles from './HeaderBarStyles';

export type HeaderBarStyle = 'default' | 'defaultDark' | 'error' | 'success';
export type HeaderBarProps = {
  style: HeaderBarStyle;
  hidden: boolean;
  showSettings: boolean;
  onSettings: ?(() => void);
};

export default class HeaderBar extends Component {
  props: HeaderBarProps;
  static defaultProps: $Shape<HeaderBarProps> = {
    style: 'default',
    hidden: false,
    showSettings: false,
    onSettings: null
  };

  render() {
    let containerClass = [
      styles['headerbar'],
      styles['headerbar__' + process.platform],
      styles['headerbar__style_' + this.props.style]
    ];

    if(this.props.hidden) {
      containerClass.push(styles['headerbar__hidden']);
    }

    return (
      <View style={ containerClass }>
        {!this.props.hidden ?
          <View style={styles.headerbar__container} testName="headerbar__container">
            <Img style={ styles.headerbar__logo } source='logo-icon'/>
            <Text style={styles.headerbar__title}>MULLVAD VPN</Text>
          </View>
          : null}

        {this.props.showSettings ?
          <Button style={ styles.headerbar__settings } onPress={ this.props.onSettings } testName="headerbar__settings">
            <Img source='icon-settings' style={ styles.headerbar__settings_icon } hoverStyle={ styles.settings_icon_hover }/>
          </Button>
          : null}
      </View>
    );
  }
}
