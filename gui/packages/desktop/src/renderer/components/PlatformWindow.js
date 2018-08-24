// @flow

import * as React from 'react';
import { Component, View, Styles } from 'reactxp';

type Props = {
  arrowPosition: number,
};

export default class PlatformWindow extends Component<Props> {
  render() {
    let style = undefined;

    if (process.platform === 'darwin') {
      const webkitMask = [
        `url(../assets/images/app-triangle.svg) ${this.props.arrowPosition}% 0% no-repeat`,
        `url(../assets/images/app-header-backdrop.svg) no-repeat`,
      ];

      style = Styles.createViewStyle({ WebkitMask: webkitMask.join(',') }, false);
    }

    return <View style={style}>{this.props.children}</View>;
  }
}
