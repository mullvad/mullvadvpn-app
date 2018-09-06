// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Clipboard, Text, View, Types } from 'reactxp';
import { Accordion } from '@mullvad/components';
import { Layout, Container, Header } from './Layout';
import { SettingsBarButton, Brand } from './HeaderBar';
import * as AppButton from './AppButton';
import Img from './Img';
import Map from './Map';
import styles from './ConnectStyles';
import { BlockedError, NoCreditError, NoInternetError } from '../errors';
import WindowStateObserver from '../lib/window-state-observer';

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

type State = {
  showCopyIPMessage: boolean,
  mapOffset: [number, number],
};

export default class Connect extends Component<Props, State> {
  state = {
    showCopyIPMessage: false,
    mapOffset: [0, 0],
  };

  _copyTimer: ?TimeoutID;
  _windowStateObserver = new WindowStateObserver();

  shouldComponentUpdate(nextProps: Props, nextState: State) {
    const { connection: prevConnection, ...otherPrevProps } = this.props;
    const { connection: nextConnection, ...otherNextProps } = nextProps;

    const prevState = this.state;

    return (
      // shallow compare the connection
      !shallowCompare(prevConnection, nextConnection) ||
      !shallowCompare(otherPrevProps, otherNextProps) ||
      prevState.mapOffset[0] !== nextState.mapOffset[0] ||
      prevState.mapOffset[1] !== nextState.mapOffset[1] ||
      prevState.showCopyIPMessage !== nextState.showCopyIPMessage
    );
  }

  componentDidMount() {
    this.props.updateAccountExpiry();

    this._windowStateObserver.onShow = () => {
      this.props.updateAccountExpiry();
    };
  }

  componentWillUnmount() {
    if (this._copyTimer) {
      clearTimeout(this._copyTimer);
    }

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

    if (error instanceof BlockedError) {
      title = 'Blocked';
      message = error.message;
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
        markerStyle: status === 'connected' ? 'secure' : 'unsecure',
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

  _updateMapOffset = (spinnerNode: ?HTMLElement) => {
    if (spinnerNode) {
      // calculate the vertical offset from the center of the map
      // to shift the center of the map upwards to align the centers
      // of spinner and marker on the map
      const y = spinnerNode.offsetTop + spinnerNode.clientHeight * 0.5;
      this.setState({
        mapOffset: [0, y],
      });
    }
  };

  renderMap() {
    let [isConnecting, isConnected, isDisconnected, isDisconnecting] = [false, false, false, false];
    switch (this.props.connection.status) {
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
    }

    return (
      <View style={styles.connect}>
        <View style={styles.map}>
          <Map style={{ width: '100%', height: '100%' }} {...this._getMapProps()} />
        </View>
        <View style={styles.container}>
          {this._renderIsBlockingInternetMessage()}

          {/* show spinner when connecting */}
          {isConnecting ? (
            <View style={styles.status_icon}>
              <Img
                source="icon-spinner"
                height={60}
                width={60}
                alt=""
                ref={this._updateMapOffset}
              />
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

          {/* footer when connecting */}
          {isConnecting ? (
            <View style={styles.footer}>
              <AppButton.TransparentButton
                style={styles.switch_location_button}
                onPress={this.props.onSelectLocation}>
                {'Switch location'}
              </AppButton.TransparentButton>
              <AppButton.RedTransparentButton onPress={this.props.onDisconnect}>
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

  _renderIsBlockingInternetMessage() {
    return (
      <Accordion
        style={styles.blocking_container}
        height={this.props.connection.status === 'connecting' ? 'auto' : 0}>
        <Text style={styles.blocking_message}>
          <Text style={styles.blocking_icon}>&nbsp;</Text>
          <Text>BLOCKING INTERNET</Text>
        </Text>
      </Accordion>
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
    switch (status) {
      case 'disconnecting':
      case 'disconnected':
        return 'error';
      case 'connecting':
      case 'connected':
      case 'blocked':
        return 'success';
      default:
        throw new Error(`Invalid ConnectionState: ${(status: empty)}`);
    }
  }

  networkSecurityStyle(): Types.Style {
    const classes = [styles.status_security];
    if (this.props.connection.status === 'connected') {
      classes.push(styles.status_security__secure);
    } else if (this.props.connection.status === 'disconnected') {
      classes.push(styles.status_security__unsecured);
    }
    return classes;
  }

  networkSecurityMessage(): string {
    switch (this.props.connection.status) {
      case 'connected':
        return 'SECURE CONNECTION';
      case 'connecting':
        return 'CREATING SECURE CONNECTION';
      default:
        return 'UNSECURED CONNECTION';
    }
  }

  ipAddressStyle(): Types.Style {
    var classes = [styles.status_ipaddress];
    if (this.props.connection.status === 'connecting') {
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

    // Tunnel is blocked due to an error?
    if (this.props.connection.status === 'blocked') {
      return new BlockedError(this.props.connection.blockReason);
    }

    return null;
  }
}

function shallowCompare(lhs: Object, rhs: Object) {
  const keys = Object.keys(lhs);
  return keys.length === Object.keys(rhs).length && keys.every((key) => lhs[key] === rhs[key]);
}
