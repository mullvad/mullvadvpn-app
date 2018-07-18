// @flow

import * as React from 'react';
import { Button, Component, Text, View, Styles } from 'reactxp';
import Img from './Img';
import { colors } from '../../config';

const styles = {
  navigationBar: {
    default: Styles.createViewStyle({
      flex: 0,
      alignItems: 'flex-start',
      paddingLeft: 12,
    }),
    darwin: Styles.createViewStyle({
      paddingTop: 24,
    }),
    win32: Styles.createViewStyle({
      paddingTop: 12,
    }),
    linux: Styles.createViewStyle({
      paddingTop: 12,
      WebkitAppRegion: 'drag',
    }),
  },
  closeBarItem: {
    default: Styles.createViewStyle({
      cursor: 'default',
      WebkitAppRegion: 'no-drag',
    }),
    icon: Styles.createViewStyle({
      flex: 0,
      opacity: 0.6,
    }),
  },
  backBarButton: {
    default: Styles.createViewStyle({
      borderWidth: 0,
      padding: 0,
      margin: 0,
      cursor: 'default',
      WebkitAppRegion: 'no-drag',
    }),
    content: Styles.createViewStyle({
      flexDirection: 'row',
      alignItems: 'center',
    }),
    label: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      color: colors.white60,
    }),
    icon: Styles.createViewStyle({
      opacity: 0.6,
      marginRight: 8,
    }),
  },
};

export default class NavigationBar extends Component {
  render() {
    return (
      <View style={[styles.navigationBar.default, styles.navigationBar[process.platform]]}>
        {this.props.children}
      </View>
    );
  }
}

export class CloseBarItem extends Component {
  props: {
    action: () => void,
  };
  render() {
    return (
      <Button style={[styles.closeBarItem.default]} onPress={this.props.action}>
        <Img height={24} width={24} style={[styles.closeBarItem.icon]} source="icon-close" />
      </Button>
    );
  }
}

export class BackBarItem extends Component {
  props: {
    title: string,
    action: () => void,
  };
  render() {
    return (
      <Button style={styles.backBarButton.default} onPress={this.props.action}>
        <View style={styles.backBarButton.content}>
          <Img style={styles.backBarButton.icon} source="icon-back" />
          <Text style={styles.backBarButton.label}>{this.props.title}</Text>
        </View>
      </Button>
    );
  }
}
