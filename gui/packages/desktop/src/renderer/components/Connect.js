// @flow

import * as React from 'react';
import { Component, View } from 'reactxp';
import { SettingsBarButton, Brand, HeaderBarStyle, ImageView } from '@mullvad/components';
import { Layout, Container, Header } from './Layout';
import NotificationArea from './NotificationArea';
import * as AppButton from './AppButton';
import TunnelControl from './TunnelControl';
import Map from './Map';
import styles from './ConnectStyles';
import { NoCreditError, NoInternetError } from '../../main/errors';

import type { RelayOutAddress, RelayInAddress } from './TunnelControl';
import type AccountExpiry from '../lib/account-expiry';
import type { ConnectionReduxState } from '../redux/connection/reducers';
import type { VersionReduxState } from '../redux/version/reducers';

type Props = {
  connection: ConnectionReduxState,
  version: VersionReduxState,
  accountExpiry: ?AccountExpiry,
  selectedRelayName: string,
  connectionInfoOpen: boolean,
  blockWhenDisconnected: boolean,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
  onToggleConnectionInfo: (boolean) => void,
};

type MarkerOrSpinner = 'marker' | 'spinner';

export default class Connect extends Component<Props> {
  render() {
    const error = this.checkForErrors();
    const child = error ? this.renderError(error) : this.renderMap();

    return (
      <Layout>
        <Header barStyle={this.headerBarStyle()}>
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
          <ImageView source="icon-fail" height={60} width={60} alt="" />
        </View>
        <View style={styles.body}>
          <View style={styles.error_title}>{title}</View>
          <View style={styles.error_message}>{message}</View>
          {error instanceof NoCreditError ? (
            <View>
              <AppButton.GreenButton onPress={() => this.props.onExternalLink('purchase')}>
                <AppButton.Label>Buy more time</AppButton.Label>
                <ImageView source="icon-extLink" height={16} width={16} />
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
        // do not show the marker when connecting or reconnecting
        showMarker: this._showMarkerOrSpinner() === 'marker',
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
            throw new Error(`Invalid action after disconnection: ${(status.details: empty)}`);
        }
      default:
        throw new Error(`Invalid connection status: ${(status.state: empty)}`);
    }
  }

  _showMarkerOrSpinner(): MarkerOrSpinner {
    const state = this.props.connection.status.state;
    const details = this.props.connection.status.details;

    if (state === 'connecting' || (state === 'disconnecting' && details === 'reconnect')) {
      return 'spinner';
    } else {
      return 'marker';
    }
  }

  renderMap() {
    const tunnelState = this.props.connection.status.state;
    const details = this.props.connection.status.details;

    const relayOutAddress: RelayOutAddress = {
      ipv4: this.props.connection.ip,
      ipv6: null,
    };
    let relayInAddress: ?RelayInAddress = null;

    if ((tunnelState === 'connecting' || tunnelState === 'connected') && details) {
      relayInAddress = {
        ip: details.address,
        port: details.tunnel.openvpn.port,
        protocol: details.tunnel.openvpn.protocol,
      };
    }

    return (
      <View style={styles.connect}>
        <View style={styles.map}>
          <Map style={{ width: '100%', height: '100%' }} {...this._getMapProps()} />
        </View>
        <View style={styles.container}>
          {/* show spinner when connecting */}
          {this._showMarkerOrSpinner() === 'spinner' ? (
            <View style={styles.status_icon}>
              <ImageView source="icon-spinner" height={60} width={60} alt="" />
            </View>
          ) : null}

          <TunnelControl
            tunnelState={this.props.connection.status}
            selectedRelayName={this.props.selectedRelayName}
            city={this.props.connection.city}
            country={this.props.connection.country}
            hostname={this.props.connection.hostname}
            defaultConnectionInfoOpen={this.props.connectionInfoOpen}
            relayInAddress={relayInAddress}
            relayOutAddress={relayOutAddress}
            onConnect={this.props.onConnect}
            onDisconnect={this.props.onDisconnect}
            onSelectLocation={this.props.onSelectLocation}
            onToggleConnectionInfo={this.props.onToggleConnectionInfo}
          />

          <NotificationArea
            style={styles.notification_area}
            tunnelState={this.props.connection.status}
            version={this.props.version}
            accountExpiry={this.props.accountExpiry}
            openExternalLink={this.props.onExternalLink}
            blockWhenDisconnected={this.props.blockWhenDisconnected}
          />
        </View>
      </View>
    );
  }

  // Private

  headerBarStyle(): HeaderBarStyle {
    const { status } = this.props.connection;
    switch (status.state) {
      case 'disconnected':
        return HeaderBarStyle.error;
      case 'connecting':
      case 'connected':
      case 'blocked':
        return HeaderBarStyle.success;
      case 'disconnecting':
        switch (status.details) {
          case 'block':
          case 'reconnect':
            return HeaderBarStyle.success;
          case 'nothing':
            return HeaderBarStyle.error;
          default:
            throw new Error(`Invalid action after disconnection: ${(status.details: empty)}`);
        }
      default:
        throw new Error(`Invalid TunnelState: ${(status.state: empty)}`);
    }
  }

  checkForErrors(): ?Error {
    // Offline?
    if (!this.props.connection.isOnline) {
      return new NoInternetError();
    }

    // No credit?
    if (this.props.accountExpiry && this.props.accountExpiry.hasExpired()) {
      return new NoCreditError();
    }

    return null;
  }
}
