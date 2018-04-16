// @flow

import * as React from 'react';
import { View, Component } from 'reactxp';

type Props = {
  autoHide: boolean,
  thumbInset: { x: number, y: number },
  children?: React.Node,
};

type State = {
  canScroll: boolean,
  showScrollIndicators: boolean,
};

export default class CustomScrollbars extends Component<Props, State> {
  static defaultProps = {
    autoHide: true,
    thumbInset: { x: 2, y: 2 },
  };

  state = {
    canScroll: false,
    showScrollIndicators: true,
  };

  render() {
    const { autoHide: _autoHide, thumbInset: _thumbInset, children, ...otherProps } = this.props;

    return (
      <View { ...otherProps }>
        { children }
      </View>);
  }
}
