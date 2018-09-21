// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Clipboard, Text, View, Types } from 'reactxp';
import { Accordion } from '@mullvad/components';
import { Layout, Container, Header } from './Layout';
import { SettingsBarButton, Brand } from './HeaderBar';
import BlockingInternetBanner, { BannerTitle, BannerSubtitle } from './BlockingInternetBanner';
import * as AppButton from './AppButton';
import Img from './Img';
import Map from './Map';
import styles from './ConnectStyles';
import { NoCreditError, NoInternetError } from '../errors';
import type { BlockReason, TunnelStateTransition } from '../lib/daemon-rpc';

import type { HeaderBarStyle } from './HeaderBar';
import type { ConnectionReduxState } from '../redux/connection/reducers';

type Props = {
  connection: ConnectionReduxState,
  accountExpiry: ?string,
  selectedRelayName: string,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
};

type State = {
  banner: {
    visible: boolean,
    title: string,
    subtitle: string,
  },
  showCopyIPMessage: boolean,
};

function getBlockReasonMessage(blockReason: BlockReason): string {
  switch (blockReason.reason) {
    case 'auth_failed': {
      const details =
        blockReason.details ||
        'Check that the account is valid, has time left and not too many connections';
      return `Authentication failed: ${details}`;
    }
    case 'ipv6_unavailable':
      return 'Could not configure IPv6, please enable it on your system or disable it in the app';
    case 'set_security_policy_error':
      return 'Failed to apply security policy';
    case 'start_tunnel_error':
      return 'Failed to start tunnel connection';
    case 'no_matching_relay':
      return 'No relay server matches the current settings';
    default:
      return `Unknown error: ${(blockReason.reason: empty)}`;
  }
}

export default class Connect extends Component<Props, State> {
  state = {
    banner: {
      visible: false,
      title: '',
      subtitle: '',
    },
    showCopyIPMessage: false,
  };

  _copyTimer: ?TimeoutID;

  constructor(props: Props) {
    super();

    const connection = props.connection;
    this.state = {
      ...this.state,
      banner: this.getBannerState(connection.status),
    };
  }

  componentWillUnmount() {
    if (this._copyTimer) {
      clearTimeout(this._copyTimer);
    }
  }

  componentDidUpdate(oldProps: Props, _oldState: State) {
    const oldConnection = oldProps.connection;
    const newConnection = this.props.connection;

    if (
      oldConnection.status.state !== newConnection.status.state ||
      oldConnection.status.details !== newConnection.status.details
    ) {
      this.setState({
        banner: this.getBannerState(newConnection.status),
      });
    }
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

  getBannerState(tunnelState: TunnelStateTransition): $PropertyType<State, 'banner'> {
    switch (tunnelState.state) {
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
          subtitle: getBlockReasonMessage(tunnelState.details),
        };

      default:
        return {
          ...this.state.banner,
          visible: false,
        };
    }
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
    const state = status.state;

    // when the user location is known
    if (typeof longitude === 'number' && typeof latitude === 'number') {
      return {
        center: [longitude, latitude],
        // do not show the marker when connecting
        showMarker: state !== 'connecting',
        markerStyle: this._getMarkerStyle(),
        // zoom in when connected
        zoomLevel: state === 'connected' ? 'low' : 'medium',
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

  _getMarkerStyle() {
    const { status } = this.props.connection;

    switch (status.state) {
      case 'connecting':
      case 'connected':
      case 'blocked':
        return 'secure';
      case 'disconnected':
        return 'unsecure';
      case 'disconnecting':
        switch (status.details) {
          case 'block':
          case 'reconnect':
            return 'secure';
          case 'nothing':
            return 'unsecure';
          default:
            throw new Error(`Invalid action after disconnection: $(status.details: empty)}`);
        }
      default:
        throw new Error(`Invalid connection status: ${(status.state: empty)}`);
    }
  }

  renderMap() {
    let [isConnecting, isConnected, isDisconnected, isDisconnecting, isBlocked] = [
      false,
      false,
      false,
      false,
      false,
    ];
    switch (this.props.connection.status.state) {
      case 'connecting':
        isConnecting = true;
        break;
      case 'connected':
        isConnected = true;
        break;
      case 'disconnected':
        isDisconnected = true;
        break;
      case 'disconnecting':
        isDisconnecting = true;
        break;
      case 'blocked':
        isBlocked = true;
        break;
    }

    return (
      <View style={styles.connect}>
        <View style={styles.map}>
          <Map style={{ width: '100%', height: '100%' }} {...this._getMapProps()} />
        </View>
        <View style={styles.container}>
          <Accordion
            style={styles.blocking_container}
            height={this.state.banner.visible ? 'auto' : 0}
            testName={'blockingAccordion'}>
            <BlockingInternetBanner>
              <BannerTitle>{this.state.banner.title}</BannerTitle>
              <BannerSubtitle>{this.state.banner.subtitle}</BannerSubtitle>
            </BlockingInternetBanner>
          </Accordion>

          {/* show spinner when connecting */}
          {isConnecting ? (
            <View style={styles.status_icon}>
              <Img source="icon-spinner" height={60} width={60} alt="" />
            </View>
          ) : null}

          <View style={styles.status}>
            <View style={this.networkSecurityStyle()} testName="networkSecurityMessage">
              {this.networkSecurityMessage()}
            </View>

            {/*
              **********************************
              Begin: Location block
              **********************************
            */}

            {/* location when connecting, disconnecting or disconnected */}
            {isConnecting || isDisconnecting || isDisconnected ? (
              <Text style={styles.status_location} testName="location">
                {this.props.connection.country}
              </Text>
            ) : null}

            {/* location when connected */}
            {isConnected ? (
              <Text style={styles.status_location} testName="location">
                {this.props.connection.city}
                {this.props.connection.city && <br />}
                {this.props.connection.country}
              </Text>
            ) : null}

            {/*
              **********************************
              End: Location block
              **********************************
            */}

            <Text style={this.ipAddressStyle()} onPress={this.onIPAddressClick.bind(this)}>
              {isConnected || isDisconnecting || isDisconnected ? (
                <Text testName="ipAddress">
                  {this.state.showCopyIPMessage
                    ? 'IP copied to clipboard!'
                    : this.props.connection.ip}
                </Text>
              ) : null}
            </Text>
          </View>

          {/*
            **********************************
            Begin: Footer block
            **********************************
          */}

          {/* footer when disconnecting or disconnected */}
          {isDisconnecting || isDisconnected ? (
            <View style={styles.footer}>
              <AppButton.TransparentButton
                style={styles.switch_location_button}
                onPress={this.props.onSelectLocation}>
                <AppButton.Label>{this.props.selectedRelayName}</AppButton.Label>
                <Img height={12} width={7} source="icon-chevron" />
              </AppButton.TransparentButton>
              <AppButton.GreenButton onPress={this.props.onConnect} testName="secureConnection">
                {'Secure my connection'}
              </AppButton.GreenButton>
            </View>
          ) : null}

          {/* footer when connecting or blocked */}
          {isConnecting || isBlocked ? (
            <View style={styles.footer}>
              <AppButton.TransparentButton
                style={styles.switch_location_button}
                onPress={this.props.onSelectLocation}>
                {'Switch location'}
              </AppButton.TransparentButton>
              <AppButton.RedTransparentButton onPress={this.props.onDisconnect} testName="cancel">
                {'Cancel'}
              </AppButton.RedTransparentButton>
            </View>
          ) : null}

          {/* footer when connected */}
          {isConnected ? (
            <View style={styles.footer}>
              <AppButton.TransparentButton
                style={styles.switch_location_button}
                onPress={this.props.onSelectLocation}>
                {'Switch location'}
              </AppButton.TransparentButton>
              <AppButton.RedTransparentButton
                onPress={this.props.onDisconnect}
                testName="disconnect">
                {'Disconnect'}
              </AppButton.RedTransparentButton>
            </View>
          ) : null}

          {/*
            **********************************
            End: Footer block
            **********************************
          */}
        </View>
      </View>
    );
  }

  // Handlers

  onExternalLink(type: string) {
    this.props.onExternalLink(type);
  }

  onIPAddressClick() {
    if (this._copyTimer) {
      clearTimeout(this._copyTimer);
    }
    this._copyTimer = setTimeout(() => this.setState({ showCopyIPMessage: false }), 3000);
    this.setState({ showCopyIPMessage: true });

    const { ip } = this.props.connection;
    if (ip) {
      Clipboard.setText(ip);
    }
  }

  // Private

  headerBarStyle(): HeaderBarStyle {
    const { status } = this.props.connection;
    switch (status.state) {
      case 'disconnected':
        return 'error';
      case 'connecting':
      case 'connected':
      case 'blocked':
        return 'success';
      case 'disconnecting':
        switch (status.details) {
          case 'block':
          case 'reconnect':
            return 'success';
          case 'nothing':
            return 'error';
          default:
            throw new Error(`Invalid action after disconnection: ${(status.details: empty)}`);
        }
      default:
        throw new Error(`Invalid TunnelState: ${(status.state: empty)}`);
    }
  }

  networkSecurityStyle(): Types.Style {
    const classes = [styles.status_security];
    const { state } = this.props.connection.status;
    if (state === 'connected' || state === 'blocked') {
      classes.push(styles.status_security__secure);
    } else if (state === 'disconnected' || state === 'disconnecting') {
      classes.push(styles.status_security__unsecured);
    }
    return classes;
  }

  networkSecurityMessage(): string {
    switch (this.props.connection.status.state) {
      case 'connected':
        return 'SECURE CONNECTION';
      case 'blocked':
        return 'BLOCKED CONNECTION';
      case 'connecting':
        return 'CREATING SECURE CONNECTION';
      default:
        return 'UNSECURED CONNECTION';
    }
  }

  ipAddressStyle(): Types.Style {
    var classes = [styles.status_ipaddress];
    if (this.props.connection.status.state === 'connecting') {
      classes.push(styles.status_ipaddress__invisible);
    }
    return classes;
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
