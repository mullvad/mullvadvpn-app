import * as React from 'react';
import { Component, Styles, Text, View } from 'reactxp';
import { colors } from '../../config.json';
import { pgettext } from '../../shared/gettext';
import { SettingsBarButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Header, Layout } from './Layout';

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

interface IProps {
  openSettings: () => void;
}

export default class Launch extends Component<IProps> {
  public render() {
    return (
      <Layout>
        <Header>
          <SettingsBarButton onPress={this.props.openSettings} />
        </Header>
        <Container>
          <View style={styles.container}>
            <ImageView height={120} width={120} source="logo-icon" style={styles.logo} />
            <Text style={styles.title}>{pgettext('launch-view', 'MULLVAD VPN')}</Text>
            <Text style={styles.subtitle}>
              {pgettext('launch-view', 'Connecting to daemon...')}
            </Text>
          </View>
        </Container>
      </Layout>
    );
  }
}
