// @flow

import * as React from 'react';
import { Component, Text, View, Styles } from 'reactxp';
import { colors } from '../../config';

const styles = {
  header: {
    default: Styles.createViewStyle({
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      paddingTop: 16,
      paddingRight: 24,
      paddingLeft: 24,
      paddingBottom: 24,
    }),
    linux: Styles.createViewStyle({
      WebkitAppRegion: 'drag',
    }),
  },
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 32,
    fontWeight: '900',
    lineHeight: 40,
    color: colors.white,
  }),
  subtitle: Styles.createTextStyle({
    marginTop: 4,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    overflow: 'visible',
    color: colors.white80,
    lineHeight: 20,
    letterSpacing: -0.2,
  }),
};

export default class SettingsHeader extends Component {
  render() {
    return (
      <View style={[styles.header.default, styles.header[process.platform], this.props.style]}>
        {this.props.children}
      </View>
    );
  }
}

export class HeaderTitle extends Component {
  render() {
    return <Text style={[styles.title]}>{this.props.children}</Text>;
  }
}

export class HeaderSubTitle extends Component {
  render() {
    return <Text style={[styles.subtitle]}>{this.props.children}</Text>;
  }
}
