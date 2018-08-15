// @flow

import * as React from 'react';
import { Component, View, Styles } from 'reactxp';

const styles = {
  darwin: Styles.createViewStyle({
    WebkitMask: `
      url(../assets/images/app-triangle.svg) 50% 0% no-repeat,
      url(../assets/images/app-header-backdrop.svg) no-repeat`,
  }),
};

export default class PlatformWindow extends Component {
  render() {
    return <View style={styles[process.platform]}>{this.props.children}</View>;
  }
}
