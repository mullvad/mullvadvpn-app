// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Clipboard, Text, View } from 'reactxp';
import { Accordion } from '@mullvad/components';
import { Layout, Container, Header } from './Layout';
import { SettingsBarButton, Brand } from './HeaderBar';
import BlockingInternetBanner, { BannerTitle, BannerSubtitle } from './BlockingInternetBanner';
import * as AppButton from './AppButton';
import Img from './Img';
import Map from './Map';
import styles from './ConnectStyles';
import { NoCreditError, NoInternetError } from '../errors';
import WindowStateObserver from '../lib/window-state-observer';
import type { BlockReason, TunnelState } from '../lib/daemon-rpc';

import type { HeaderBarStyle } from './HeaderBar';
import type { ConnectionReduxState } from '../redux/connection/reducers';

type Props = {
  connection: ConnectionReduxState,
  accountExpiry: string,
  selectedRelayName: string,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
  updateAccountExpiry: () => Promise<void>,
};

function getBlockReasonMessage(reason: BlockReason): string {
  switch (reason) {
    case 'ipv6_unavailable':
      return 'Could not configure IPv6, please enable it on your system or disable it in the app';
    case 'set_security_policy_error':
      return 'Failed to apply security policy';
    case 'start_tunnel_error':
      return 'Failed to start tunnel connection';
    case 'no_matching_relay':
      return 'No relay server matches the current settings';
    default:
      return `Unknown error: ${(reason: empty)}`;
  }
}

export default class Connect extends Component<Props> {
  state = {
    banner: {
      visible: false,
      title: '',
      subtitle: '',
    },
  };

  _windowStateObserver = new WindowStateObserver();

  componentDidMount() {
    this.props.updateAccountExpiry();

    this._windowStateObserver.onShow = () => {
      this.props.updateAccountExpiry();
    };
  }

  componentWillUnmount() {
    this._windowStateObserver.dispose();
  }

  render() {
    const error = this.checkForErrors();
    const child = error ? this.renderError(error) : this.renderMap();

    return (
      <Layout>
        <Header barStyle={this.headerBarStyle()} testName="header">
          <Brand />
          <SettingsBarButton onPress={this.props.onSettings} />
        </Header>
        <Container>{child}</Container>
      </Layout>
    );
  }

  renderError(error: Error) {
    let title = '';
    let message = '';

    if (error instanceof NoCreditError) {
      title = 'Out of time';
      message = 'Buy more time, so you can continue using the internet securely';
    }

    if (error instanceof NoInternetError) {
      title = 'Offline';
      message = 'Your internet connection will be secured when you get back online';
    }

    return (
      <View style={styles.connect}>
        <View style={styles.status_icon}>
          <Img source="icon-fail" height={60} width={60} alt="" />
        </View>
        <View style={styles.status}>
          <View style={styles.error_title}>{title}</View>
          <View style={styles.error_message}>{message}</View>
          {error instanceof NoCreditError ? (
            <View>
              <AppButton.GreenButton onPress={this.onExternalLink.bind(this, 'purchase')}>
                <AppButton.Label>Buy more time</AppButton.Label>
                <Img source="icon-extLink" height={16} width={16} />
              </AppButton.GreenButton>
            </View>
          ) : null}
        </View>
      </View>
    );
  }

  _getMapProps() {
    const { longitude, latitude, status } = this.props.connection;

    // when the user location is known
    if (typeof longitude === 'number' && typeof latitude === 'number') {
      return {
        center: [longitude, latitude],
        // do not show the marker when connecting
        showMarker: status !== 'connecting',
        markerStyle: status === 'connected' || status === 'blocked' ? 'secure' : 'unsecure',
        // zoom in when connected
        zoomLevel: status === 'connected' ? 'low' : 'medium',
        // a magic offset to align marker with spinner
        offset: [0, 123],
      };
    } else {
      return {
        center: [0, 0],
        showMarker: false,
        markerStyle: 'unsecure',
        // show the world when user location is not known
        zoomLevel: 'high',
        // remove the offset since the marker is hidden
        offset: [0, 0],
      };
    }
  }

  renderMap() {
    return (
      <View style={styles.connect}>
        <View style={styles.map}>
          <Map style={{ width: '100%', height: '100%' }} {...this._getMapProps()} />
        </View>
        <View style={styles.container}>
          <TunnelBanner
            tunnelState={this.props.connection.status}
            blockReason={this.props.connection.blockReason}
          />

          {/* show spinner when connecting */}
          {this.props.connection.status === 'connecting' ? (
            <View style={styles.status_icon}>
              <Img source="icon-spinner" height={60} width={60} alt="" />
            </View>
          ) : null}

          <TunnelControl
            style={styles.tunnel_control}
            tunnelState={this.props.connection.status}
            selectedRelayName={this.props.selectedRelayName}
            city={this.props.connection.city}
            country={this.props.connection.country}
            ip={this.props.connection.ip}
            onConnect={this.props.onConnect}
            onDisconnect={this.props.onDisconnect}
            onSelectLocation={this.props.onSelectLocation}
          />
        </View>
      </View>
    );
  }

  // Handlers

  onExternalLink(type: string) {
    this.props.onExternalLink(type);
  }

  // Private

  headerBarStyle(): HeaderBarStyle {
    const { status } = this.props.connection;
    switch (status) {
      case 'disconnecting':
      case 'disconnected':
        return 'error';
      case 'connecting':
      case 'connected':
      case 'blocked':
        return 'success';
      default:
        throw new Error(`Invalid TunnelState: ${(status: empty)}`);
    }
  }

  checkForErrors(): ?Error {
    // Offline?
    if (!this.props.connection.isOnline) {
      return new NoInternetError();
    }

    // No credit?
    const expiry = this.props.accountExpiry;
    if (expiry && moment(expiry).isSameOrBefore(moment())) {
      return new NoCreditError();
    }

    return null;
  }
}

class TunnelBanner extends Component<
  {
    tunnelState: TunnelState,
    blockReason: ?BlockReason,
  },
  {
    visible: boolean,
    title: string,
    subtitle: string,
  },
> {
  state = {
    visible: false,
    title: '',
    subtitle: '',
  };

  constructor(props) {
    super();
    this.state = this._deriveState(props.tunnelState, props.blockReason);
  }

  componentDidUpdate(oldProps, _oldState) {
    if (
      oldProps.tunnelState !== this.props.tunnelState ||
      oldProps.blockReason !== this.props.blockReason
    ) {
      const nextState = this._deriveState(this.props.tunnelState, this.props.blockReason);
      this.setState(nextState);
    }
  }

  render() {
    return (
      <Accordion
        style={styles.blocking_container}
        height={this.state.visible ? 'auto' : 0}
        testName={'blockingAccordion'}>
        <BlockingInternetBanner>
          <BannerTitle>{this.state.title}</BannerTitle>
          <BannerSubtitle>{this.state.subtitle}</BannerSubtitle>
        </BlockingInternetBanner>
      </Accordion>
    );
  }

  _deriveState(tunnelState: TunnelState, blockReason: ?BlockReason) {
    switch (tunnelState) {
      case 'connecting':
        return {
          visible: true,
          title: 'BLOCKING INTERNET',
          subtitle: '',
        };

      case 'blocked':
        return {
          visible: true,
          title: 'BLOCKING INTERNET',
          subtitle: blockReason ? getBlockReasonMessage(blockReason) : '',
        };

      default:
        return {
          ...this.state,
          visible: false,
        };
    }
  }
}

type SecuredDisplayStyle = 'secured' | 'blocked' | 'securing' | 'unsecured';
class SecuredLabel extends Component<{
  displayStyle: SecuredDisplayStyle,
}> {
  _getText() {
    switch (this.props.displayStyle) {
      case 'secured':
        return 'SECURE CONNECTION';

      case 'blocked':
        return 'BLOCKED CONNECTION';

      case 'securing':
        return 'CREATING SECURE CONNECTION';

      case 'unsecured':
        return 'UNSECURED CONNECTION';

      default:
        throw new Error(`Unknown SecuredDisplayStyle: ${(this.props.displayStyle: empty)}`);
    }
  }

  _getTextStyle() {
    switch (this.props.displayStyle) {
      case 'secured':
      case 'blocked':
        return styles.status_security__secure;

      case 'securing':
        return styles.status_security__securing;

      case 'unsecured':
        return styles.status_security__unsecured;

      default:
        throw new Error(`Unknown SecuredDisplayStyle: ${(this.props.displayStyle: empty)}`);
    }
  }

  render() {
    return (
      <Text
        style={[styles.status_security, this._getTextStyle()]}
        testName={'networkSecurityMessage'}>
        {this._getText()}
      </Text>
    );
  }
}

class IpAddressLabel extends Component<
  {
    value: string,
  },
  {
    showsMessage: boolean,
  },
> {
  _timer: ?TimeoutID;

  state = {
    showsMessage: false,
  };

  componentWillUnmount() {
    if (this._timer) {
      clearTimeout(this._timer);
    }
  }

  render() {
    return (
      <Text style={styles.status_ipaddress} onPress={this._handlePress}>
        {this.state.showsMessage ? 'IP copied to clipboard!' : this.props.value}
      </Text>
    );
  }

  _handlePress = () => {
    if (this._timer) {
      clearTimeout(this._timer);
    }
    this._timer = setTimeout(() => this.setState({ showsMessage: false }), 3000);
    this.setState({ showsMessage: true });
    Clipboard.setText(this.props.value);
  };
}

class TunnelControl extends Component<{
  tunnelState: TunnelState,
  selectedRelayName: string,
  city: ?string,
  country: ?string,
  ip: ?string,
  onConnect: () => void,
  onDisconnect: () => void,
  onSelectLocation: () => void,
}> {
  render() {
    const Location = ({ children }) => <View style={styles.status_location}>{children}</View>;
    const City = () => <Text style={styles.status_location_text}>{this.props.city}</Text>;
    const Country = () => <Text style={styles.status_location_text}>{this.props.country}</Text>;
    const Ip = () => <IpAddressLabel value={this.props.ip || ''} />;

    const SwitchLocation = () => {
      return (
        <AppButton.TransparentButton
          style={styles.switch_location_button}
          onPress={this.props.onSelectLocation}>
          <AppButton.Label>{'Switch location'}</AppButton.Label>
        </AppButton.TransparentButton>
      );
    };

    const SelectedLocation = () => (
      <AppButton.TransparentButton
        style={styles.switch_location_button}
        onPress={this.props.onSelectLocation}>
        <AppButton.Label>{this.props.selectedRelayName}</AppButton.Label>
        <Img height={12} width={7} source="icon-chevron" />
      </AppButton.TransparentButton>
    );

    const Connect = () => (
      <AppButton.GreenButton onPress={this.props.onConnect} testName="secureConnection">
        {'Secure my connection'}
      </AppButton.GreenButton>
    );

    const Disconnect = () => (
      <AppButton.RedTransparentButton onPress={this.props.onDisconnect} testName="disconnect">
        {'Disconnect'}
      </AppButton.RedTransparentButton>
    );

    const Cancel = () => (
      <AppButton.RedTransparentButton onPress={this.props.onDisconnect} testName="cancel">
        {'Cancel'}
      </AppButton.RedTransparentButton>
    );

    const Wrapper = ({ children }) => <View style={this.props.style}>{children}</View>;
    const Body = ({ children }) => <View style={styles.status}>{children}</View>;
    const Footer = ({ children }) => <View style={styles.footer}>{children}</View>;

    switch (this.props.tunnelState) {
      case 'connecting':
        return (
          <Wrapper>
            <Body>
              <SecuredLabel displayStyle={'securing'} />
              <Location>
                <City />
              </Location>
            </Body>
            <Footer>
              <SwitchLocation />
              <Cancel />
            </Footer>
          </Wrapper>
        );
      case 'connected':
        return (
          <Wrapper>
            <Body>
              <SecuredLabel displayStyle={'secured'} />
              <Location>
                <City />
                <Country />
              </Location>
              <Ip />
            </Body>
            <Footer>
              <SwitchLocation />
              <Disconnect />
            </Footer>
          </Wrapper>
        );

      case 'blocked':
        return (
          <Wrapper>
            <Body>
              <SecuredLabel displayStyle={'blocked'} />
            </Body>
            <Footer>
              <SwitchLocation />
              <Cancel />
            </Footer>
          </Wrapper>
        );

      case 'disconnecting':
        return (
          <Wrapper>
            <Body>
              <SecuredLabel displayStyle={'secured'} />
              <Location>
                <Country />
              </Location>
              <Ip />
            </Body>
            <Footer>
              <SelectedLocation />
              <Connect />
            </Footer>
          </Wrapper>
        );

      case 'disconnected':
        return (
          <Wrapper>
            <Body>
              <SecuredLabel displayStyle={'unsecured'} />
              <Location>
                <Country />
              </Location>
              <Ip />
            </Body>
            <Footer>
              <SelectedLocation />
              <Connect />
            </Footer>
          </Wrapper>
        );

      default:
        throw new Error(`Unknown TunnelState: ${(this.props.tunnelState: empty)}`);
    }
  }
}
