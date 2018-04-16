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
      styles[process.platform],
      styles['style_' + this.props.style]
    ];

    if(this.props.hidden) {
      containerClass.push(styles['hidden']);
    }

    return (
      <View style={ containerClass }>
        {!this.props.hidden ?
          <View style={styles.container} testName="headerbar__container">
            <Img height={50} width={50} source='logo-icon'/>
            <Text style={styles.title}>MULLVAD VPN</Text>
          </View>
          : null}

        {this.props.showSettings ?
          <Button style={ styles.settings } onPress={ this.props.onSettings } testName="headerbar__settings">
            <Img height={24} width={24} source='icon-settings' style={ styles.settings_icon } hoverStyle={ styles.settings_icon_hover }/>
          </Button>
          : null}
      </View>
    );
  }
}
