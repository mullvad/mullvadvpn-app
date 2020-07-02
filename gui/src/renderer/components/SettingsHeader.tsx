import * as React from 'react';
import { Component, Styles, Text, Types, View } from 'reactxp';
import { colors } from '../../config.json';

const styles = {
  header: {
    default: Styles.createViewStyle({
      flex: 0,
      paddingTop: 2,
      paddingRight: 20,
      paddingLeft: 20,
      paddingBottom: 20,
    }),
  },
  // TODO: Use bigText in comonStyles when converted from ReactXP
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 30,
    fontWeight: '900',
    lineHeight: 34,
    color: colors.white,
  }),
  // TODO: Use smallText in comonStyles when converted from ReactXP
  subtitle: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    overflow: 'visible',
    color: colors.white80,
    lineHeight: 20,
  }),
  spacer: Styles.createViewStyle({
    height: 8,
  }),
};

interface ISettingsHeaderProps {
  style?: Types.ViewStyleRuleSet;
}

interface ISettingsTextProps {
  style?: Types.TextStyleRuleSet;
}

export default class SettingsHeader extends Component<ISettingsHeaderProps> {
  public render() {
    return (
      <View style={[styles.header.default, this.props.style]}>
        {React.Children.map(this.props.children, (child, index) => {
          if (React.isValidElement(child) && index > 0) {
            return (
              <React.Fragment>
                <View style={styles.spacer} />
                {child}
              </React.Fragment>
            );
          } else {
            return child;
          }
        })}
      </View>
    );
  }
}

export class HeaderTitle extends Component<ISettingsTextProps> {
  public render() {
    return <Text style={[styles.title, this.props.style]}>{this.props.children}</Text>;
  }
}

export class HeaderSubTitle extends Component<ISettingsTextProps> {
  public render() {
    return <Text style={[styles.subtitle, this.props.style]}>{this.props.children}</Text>;
  }
}
