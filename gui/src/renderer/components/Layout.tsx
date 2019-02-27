import * as React from 'react';
import { Component, View } from 'reactxp';
import HeaderBar from './HeaderBar';
import styles from './LayoutStyles';

export class Header extends Component<HeaderBar['props']> {
  public static defaultProps = HeaderBar.defaultProps;

  public render() {
    return (
      <View style={[styles.header, this.props.style]}>
        <HeaderBar barStyle={this.props.barStyle}>{this.props.children}</HeaderBar>
      </View>
    );
  }
}

interface IContainerProps {
  children: React.ReactNode;
}
export class Container extends Component<IContainerProps> {
  public render() {
    return <View style={styles.container}>{this.props.children}</View>;
  }
}

interface ILayoutProps {
  children: React.ReactNode;
}
export class Layout extends Component<ILayoutProps> {
  public render() {
    return <View style={styles.layout}>{this.props.children}</View>;
  }
}
