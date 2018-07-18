// @flow

import React from 'react';
import { Component, Text, Button, View, Styles } from 'reactxp';

import Img from './Img';
import { colors } from '../../config';

export type HeaderBarStyle = 'default' | 'defaultDark' | 'error' | 'success';
type HeaderBarProps = {
  barStyle: HeaderBarStyle,
};

const headerBarStyles = {
  container: {
    base: Styles.createViewStyle({
      paddingTop: 12,
      paddingBottom: 12,
      paddingLeft: 12,
      paddingRight: 12,
    }),
    platformOverride: {
      darwin: Styles.createViewStyle({
        paddingTop: 24,
      }),
      linux: Styles.createViewStyle({
        WebkitAppRegion: 'drag',
      }),
    },
  },
  content: Styles.createViewStyle({
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-end',
    // the size of "brand" logo
    minHeight: 51,
  }),
  barStyle: {
    default: Styles.createViewStyle({
      backgroundColor: colors.blue,
    }),
    defaultDark: Styles.createViewStyle({
      backgroundColor: colors.darkBlue,
    }),
    error: Styles.createViewStyle({
      backgroundColor: colors.red,
    }),
    success: Styles.createViewStyle({
      backgroundColor: colors.green,
    }),
  },
};

export default class HeaderBar extends Component<HeaderBarProps> {
  static defaultProps: HeaderBarProps = {
    barStyle: 'default',
  };

  render() {
    const style = [
      headerBarStyles.container.base,
      headerBarStyles.container.platformOverride[process.platform],
      headerBarStyles.barStyle[this.props.barStyle],
      this.props.style,
    ];

    return (
      <View style={style}>
        <View style={headerBarStyles.content}>{this.props.children}</View>
      </View>
    );
  }
}

const brandStyles = {
  container: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  }),
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 24,
    fontWeight: '900',
    lineHeight: 30,
    letterSpacing: -0.5,
    color: colors.white60,
    marginLeft: 8,
  }),
};

export class Brand extends Component {
  render() {
    return (
      <View style={brandStyles.container} testName="headerbar__container">
        <Img width={50} height={50} source="logo-icon" />
        <Text style={brandStyles.title}>{'MULLVAD VPN'}</Text>
      </View>
    );
  }
}

type SettingsButtonProps = {
  onPress: ?() => void,
};

const settingsBarButtonStyles = {
  container: {
    base: Styles.createViewStyle({
      cursor: 'default',
      padding: 0,
      marginLeft: 8,
    }),
    platformOverride: {
      linux: Styles.createViewStyle({
        WebkitAppRegion: 'no-drag',
      }),
    },
  },
  icon: {
    normal: Styles.createViewStyle({
      color: colors.white60,
    }),
    hover: Styles.createViewStyle({
      color: colors.white,
    }),
  },
};

export class SettingsBarButton extends Component<SettingsButtonProps> {
  render() {
    return (
      <Button
        style={[
          settingsBarButtonStyles.container.base,
          settingsBarButtonStyles.container.platformOverride[process.platform],
        ]}
        onPress={this.props.onPress}
        testName="headerbar__settings">
        <Img
          height={24}
          width={24}
          source="icon-settings"
          style={settingsBarButtonStyles.icon.normal}
          hoverStyle={settingsBarButtonStyles.icon.hover}
        />
      </Button>
    );
  }
}
