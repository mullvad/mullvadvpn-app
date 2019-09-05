import * as React from 'react';
import { Component, Styles, Text, Types, View } from 'reactxp';
import { colors } from '../../config.json';

const styles = {
  header: {
    default: Styles.createViewStyle({
      flex: 0,
      paddingTop: 4,
      paddingRight: 24,
      paddingLeft: 24,
      paddingBottom: 24,
    }),
  },
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 32,
    fontWeight: '900',
    lineHeight: 36,
    color: colors.white,
  }),
  subtitle: Styles.createTextStyle({
    marginTop: 8,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    overflow: 'visible',
    color: colors.white80,
    lineHeight: 20,
    letterSpacing: -0.2,
  }),
};

interface ISettingsHeaderProps {
  style?: Types.ViewStyleRuleSet;
}

export default class SettingsHeader extends Component<ISettingsHeaderProps> {
  public render() {
    return <View style={[styles.header.default, this.props.style]}>{this.props.children}</View>;
  }
}

export class HeaderTitle extends Component {
  public render() {
    return <Text style={[styles.title]}>{this.props.children}</Text>;
  }
}

export class HeaderSubTitle extends Component {
  public render() {
    return <Text style={[styles.subtitle]}>{this.props.children}</Text>;
  }
}
