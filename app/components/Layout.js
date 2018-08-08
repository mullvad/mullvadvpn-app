// @flow

import * as React from 'react';
import { View, Component } from 'reactxp';
import HeaderBar from './HeaderBar';
import styles from './LayoutStyles';

export class Header extends Component<React.ElementProps<typeof HeaderBar>> {
  static defaultProps = HeaderBar.defaultProps;

  render() {
    return (
      <View style={[styles.header, this.props.style]}>
        <HeaderBar barStyle={this.props.barStyle}>{this.props.children}</HeaderBar>
      </View>
    );
  }
}

type ContainerProps = {
  children: React.Node,
};
export class Container extends Component<ContainerProps> {
  render() {
    return <View style={styles.container}>{this.props.children}</View>;
  }
}

type LayoutProps = {
  children: Array<React.Node> | React.Node,
};
export class Layout extends Component<LayoutProps> {
  render() {
    return <View style={styles.layout}>{this.props.children}</View>;
  }
}
