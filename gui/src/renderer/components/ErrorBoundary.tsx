import log from 'electron-log';
import * as React from 'react';
import { Component, Styles, Text, View } from 'reactxp';
import { colors, links } from '../../config.json';
import { messages } from '../../shared/gettext';
import PlatformWindowContainer from '../containers/PlatformWindowContainer';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';

interface IProps {
  children?: React.ReactNode;
}

interface IState {
  hasError: boolean;
}

const styles = {
  container: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: colors.blue,
  }),
  logo: Styles.createViewStyle({
    marginBottom: 5,
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
    marginHorizontal: 20,
    textAlign: 'center',
  }),
  email: Styles.createTextStyle({
    fontWeight: '900',
  }),
};

export default class ErrorBoundary extends Component<IProps, IState> {
  public state = { hasError: false };

  public componentDidCatch(error: Error, info: React.ErrorInfo) {
    this.setState({ hasError: true });

    log.error(
      `The error boundary caught an error: ${error.message}\nError stack: ${
        error.stack || 'Not available'
      }\nComponent stack: ${info.componentStack}`,
    );
  }

  public render() {
    if (this.state.hasError) {
      const reachBackMessage: React.ReactNodeArray =
        // TRANSLATORS: The message displayed to the user in case of critical error in the GUI
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(email)s - support email
        messages
          .pgettext('error-boundary-view', 'Something went wrong. Please contact us at %(email)s')
          .split('%(email)s', 2);
      reachBackMessage.splice(1, 0, <Text style={styles.email}>{links.supportEmail}</Text>);

      return (
        <PlatformWindowContainer>
          <Layout>
            <Container>
              <View style={styles.container}>
                <ImageView height={106} width={106} source="logo-icon" style={styles.logo} />
                <Text style={styles.title}>{messages.pgettext('generic', 'MULLVAD VPN')}</Text>
                <Text style={styles.subtitle}>{reachBackMessage}</Text>
              </View>
            </Container>
          </Layout>
        </PlatformWindowContainer>
      );
    } else {
      return this.props.children;
    }
  }
}
