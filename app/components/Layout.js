// @flow
import * as React from 'react';
import HeaderBar from './HeaderBar';
import { View, Component } from 'reactxp';

import type { HeaderBarProps } from './HeaderBar';

import styles from './LayoutStyles';

export class Header extends Component {
  props: HeaderBarProps;
  static defaultProps = HeaderBar.defaultProps;

  render() {
    return (
      <View style={styles.header}>
        <HeaderBar { ...this.props } />
      </View>
    );
  }
}

export class Container extends Component {
  props: {
    children: React.Node
  }

  render() {
    return (
      <View style={styles.container}>
        { this.props.children }
      </View>
    );
  }
}

export class Layout extends Component {
  props: {
    children: Array<React.Node> | React.Node
  }

  render() {
    return (
      <View style={styles.layout}>
        { this.props.children }
      </View>
    );
  }
}
