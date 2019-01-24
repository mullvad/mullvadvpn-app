import * as React from 'react';
import { Component, View } from 'reactxp';
import { SettingsBarButton, Brand, HeaderBarStyle, ImageView } from '@mullvad/components';
import { Layout, Container, Header } from './Layout';
import NotificationArea from './NotificationArea';
import * as AppButton from './AppButton';
import TunnelControl from './TunnelControl';
import Map, { MarkerStyle, ZoomLevel } from './Map';
import styles from './ConnectStyles';
import { NoCreditError, NoInternetError } from '../../main/errors';
import { TunnelEndpoint, parseSocketAddress } from '../../shared/daemon-rpc-types';
import { links } from '../../config.json';

import { RelayOutAddress, RelayInAddress } from './TunnelControl';
import AccountExpiry from '../lib/account-expiry';
import { ConnectionReduxState } from '../redux/connection/reducers';
import { VersionReduxState } from '../redux/version/reducers';

type Props = {
  connection: ConnectionReduxState;
  version: VersionReduxState;
  accountExpiry?: AccountExpiry;
  selectedRelayName: string;
  connectionInfoOpen: boolean;
  blockWhenDisconnected: boolean;
  onSettings: () => void;
  onSelectLocation: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  onExternalLink: (url: string) => void;
  onToggleConnectionInfo: (value: boolean) => void;
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

    const { isBlocked } = this.props.connection;

    return (
      <View style={styles.connect}>
        <View style={styles.status_icon}>
          <ImageView source="icon-fail" height={60} width={60} />
        </View>
        <View style={styles.body}>
          <View style={styles.error_title}>{title}</View>
          <View style={styles.error_message}>{message}</View>
          {error instanceof NoCreditError ? (
            <View>
              <AppButton.GreenButton
                disabled={isBlocked}
                onPress={() => this.props.onExternalLink(links.purchase)}>
                <AppButton.Label>Buy more time</AppButton.Label>
                <AppButton.Icon source="icon-extLink" height={16} width={16} />
              </AppButton.GreenButton>
            </View>
          ) : null}
        </View>
      </View>
    );
  }

  _getMapProps(): Map['props'] {
    const {
      longitude,
      latitude,
      status: { state },
    } = this.props.connection;

    // when the user location is known
    if (typeof longitude === 'number' && typeof latitude === 'number') {
      return {
        center: [longitude, latitude],
        // do not show the marker when connecting or reconnecting
        showMarker: this._showMarkerOrSpinner() === 'marker',
        markerStyle: this._getMarkerStyle(),
        // zoom in when connected
        zoomLevel: state === 'connected' ? ZoomLevel.low : ZoomLevel.medium,
        // a magic offset to align marker with spinner
        offset: [0, 123],
      };
    } else {
      return {
        center: [0, 0],
        showMarker: false,
        markerStyle: MarkerStyle.unsecure,
        // show the world when user location is not known
        zoomLevel: ZoomLevel.high,
        // remove the offset since the marker is hidden
        offset: [0, 0],
      };
    }
  }

  _getMarkerStyle(): MarkerStyle {
    const { status } = this.props.connection;

    switch (status.state) {
      case 'connecting':
      case 'connected':
        return MarkerStyle.secure;
      case 'blocked':
        switch (status.details.reason) {
          case 'set_firewall_policy_error':
            return MarkerStyle.unsecure;
          default:
            return MarkerStyle.secure;
        }
      case 'disconnected':
        return MarkerStyle.unsecure;
      case 'disconnecting':
        switch (status.details) {
          case 'block':
          case 'reconnect':
            return MarkerStyle.secure;
          case 'nothing':
            return MarkerStyle.unsecure;
          default:
            throw new Error(`Invalid action after disconnection: ${status.details}`);
        }
    }
  }

  _showMarkerOrSpinner(): MarkerOrSpinner {
    const status = this.props.connection.status;

    return status.state === 'connecting' ||
      (status.state === 'disconnecting' && status.details === 'reconnect')
      ? 'spinner'
      : 'marker';
  }

  _tunnelEndpointToRelayInAddress(tunnelEndpoint: TunnelEndpoint): RelayInAddress {
    const socketAddr = parseSocketAddress(tunnelEndpoint.address);
    return {
      ip: socketAddr.host,
      port: socketAddr.port,
      protocol: tunnelEndpoint.protocol,
    };
  }

  renderMap() {
    const status = this.props.connection.status;

    const relayOutAddress: RelayOutAddress = {
      ipv4: this.props.connection.ip,
    };
    const relayInAddress: RelayInAddress | undefined =
      (status.state === 'connecting' || status.state === 'connected') && status.details
        ? this._tunnelEndpointToRelayInAddress(status.details)
        : undefined;

    return (
      <View style={styles.connect}>
        <Map style={styles.map} {...this._getMapProps()} />
        <View style={styles.container}>
          {/* show spinner when connecting */}
          {this._showMarkerOrSpinner() === 'spinner' ? (
            <View style={styles.status_icon}>
              <ImageView source="icon-spinner" height={60} width={60} />
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
        return HeaderBarStyle.success;
      case 'blocked':
        switch (status.details.reason) {
          case 'set_firewall_policy_error':
            return HeaderBarStyle.error;
          default:
            return HeaderBarStyle.success;
        }
      case 'disconnecting':
        switch (status.details) {
          case 'block':
          case 'reconnect':
            return HeaderBarStyle.success;
          case 'nothing':
            return HeaderBarStyle.error;
          default:
            throw new Error(`Invalid action after disconnection: ${status.details}`);
        }
    }
  }

  checkForErrors(): Error | undefined {
    // Offline?
    if (!this.props.connection.isOnline) {
      return new NoInternetError();
    }

    // No credit?
    if (this.props.accountExpiry && this.props.accountExpiry.hasExpired()) {
      return new NoCreditError();
    }

    return undefined;
  }
}
