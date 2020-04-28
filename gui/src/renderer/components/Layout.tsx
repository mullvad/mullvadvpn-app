import * as React from 'react';
import { Component, View } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
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

export const Container = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  backgroundColor: colors.blue,
  overflow: 'hidden',
});

export const Layout = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  height: '100vh',
});
