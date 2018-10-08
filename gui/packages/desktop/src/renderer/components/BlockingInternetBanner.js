// @flow

import * as React from 'react';
import { View, Text, Component, Styles } from 'reactxp';
import { colors } from '../../config';

const styles = {
  container: Styles.createViewStyle({
    flexDirection: 'row',
    backgroundColor: 'rgba(25, 38, 56, 0.95)',
    paddingTop: 8,
    paddingLeft: 20,
    paddingRight: 20,
    paddingBottom: 8,
  }),
  icon: Styles.createViewStyle({
    width: 10,
    height: 10,
    flex: 0,
    borderRadius: 5,
    marginTop: 4,
    marginRight: 8,
    backgroundColor: colors.red,
  }),
  textContainer: Styles.createViewStyle({
    flex: 1,
  }),
  title: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 12,
    fontWeight: '800',
    lineHeight: 17,
    color: colors.white60,
  }),
  subtitle: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 12,
    fontWeight: '800',
    lineHeight: 17,
    color: colors.white40,
  }),
};

export class BannerTitle extends Component {
  render() {
    return <Text style={styles.title}>{this.props.children}</Text>;
  }
}

export class BannerSubtitle extends Component {
  render() {
    return React.Children.count(this.props.children) > 0 ? (
      <Text style={styles.subtitle}>{this.props.children}</Text>
    ) : null;
  }
}

export default class BlockingInternetBanner extends Component<{
  children: Array<React.Element<typeof BannerTitle> | React.Element<typeof BannerSubtitle>>,
}> {
  render() {
    return (
      <View style={styles.container}>
        <View style={styles.icon} />
        <View style={styles.textContainer}>{this.props.children}</View>
      </View>
    );
  }
}
