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
          <View style={styles.headerbar__container}>
            <Img style={ styles.headerbar__logo } source='logo-icon'/>
            <Text style={styles.headerbar__title}>MULLVAD VPN</Text>
          </View>
          : null}

        {this.props.showSettings ?
          <View style={styles.headerbar__settings}>
            <Button  onPress={ this.props.onSettings }>
              <Img style={ styles.headerbar__settings } source='icon-settings'/>
            </Button>
          </View>
          : null}
      </View>
    );
  }
}
