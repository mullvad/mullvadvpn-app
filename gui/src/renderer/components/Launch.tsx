import * as React from 'react';
import { Styles, Text, View } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { HeaderBarSettingsButton } from './HeaderBar';
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
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 24,
    fontWeight: '900',
    lineHeight: 30,
    color: colors.white60,
    marginBottom: 4,
  }),
  subtitle: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 14,
    lineHeight: 20,
    marginHorizontal: 22,
    color: colors.white40,
    textAlign: 'center',
  }),
};

const Logo = styled(ImageView)({
  marginBottom: '5px',
});

export default function Launch() {
  return (
    <Layout>
      <Header>
        <HeaderBarSettingsButton />
      </Header>
      <Container>
        <View style={styles.container}>
          <Logo height={106} width={106} source="logo-icon" />
          <Text style={styles.title}>{messages.pgettext('generic', 'MULLVAD VPN')}</Text>
          <Text style={styles.subtitle}>
            {messages.pgettext('launch-view', 'Connecting to Mullvad system service...')}
          </Text>
        </View>
      </Container>
    </Layout>
  );
}
