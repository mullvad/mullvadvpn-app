// @flow

import * as React from 'react';
import { Component, Styles, View, Text } from 'reactxp';
import { Layout, Container, Header } from './Layout';
import { SettingsBarButton } from './HeaderBar';
import Img from './Img';
import { colors } from '../../config';

const styles = {
  container: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    marginTop: -150,
  }),
  logo: Styles.createViewStyle({
    marginBottom: 4,
  }),
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 24,
    fontWeight: '900',
    lineHeight: 30,
    letterSpacing: -0.5,
    color: colors.white60,
    marginBottom: 4,
  }),
  subtitle: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 14,
    lineHeight: 20,
    color: colors.white40,
  }),
};

type Props = {
  openSettings: () => void,
};

export default class Launch extends Component<Props> {
  render() {
    return (
      <Layout>
        <Header>
          <SettingsBarButton onPress={this.props.openSettings} />
        </Header>
        <Container>
          <View style={styles.container} testName="headerbar__container">
            <Img height={120} width={120} source="logo-icon" style={styles.logo} />
            <Text style={styles.title}>{'MULLVAD VPN'}</Text>
            <Text style={styles.subtitle}>{'Connecting to daemon...'}</Text>
          </View>
        </Container>
      </Layout>
    );
  }
}
