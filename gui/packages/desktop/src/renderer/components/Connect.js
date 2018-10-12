// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Text, View, Types } from 'reactxp';
import { ConnectionInfo, SecuredLabel, SecuredDisplayStyle } from '@mullvad/components';
import { Layout, Container, Header } from './Layout';
import { SettingsBarButton, Brand } from './HeaderBar';
import NotificationArea from './NotificationArea';
import * as AppButton from './AppButton';
import Img from './Img';
import Map from './Map';
import styles from './ConnectStyles';
import { NoCreditError, NoInternetError } from '../errors';
import type { TunnelState } from '../lib/daemon-rpc';

import type { HeaderBarStyle } from './HeaderBar';
import type { ConnectionReduxState } from '../redux/connection/reducers';
import type { VersionReduxState } from '../redux/version/reducers';

type Props = {
  connection: ConnectionReduxState,
  version: VersionReduxState,
  accountExpiry: ?string,
  selectedRelayName: string,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
};

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
          <Img source="icon-fail" height={60} width={60} alt="" />
        </View>
        <View style={styles.body}>
          <View style={styles.error_title}>{title}</View>
          <View style={styles.error_message}>{message}</View>
          {error instanceof NoCreditError ? (
            <View>
              <AppButton.GreenButton onPress={() => this.props.onExternalLink('purchase')}>
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
            throw new Error(`Invalid action after disconnection: ${(status.details: empty)}`);
        }
      default:
        throw new Error(`Invalid connection status: ${(status.state: empty)}`);
    }
  }

  renderMap() {
    const tunnelState = this.props.connection.status.state;
    const details = this.props.connection.status.details;

    let relayIp = null;
    let relayPort = null;
    let relayProtocol = null;

    if ((tunnelState === 'connecting' || tunnelState === 'connected') && details) {
      relayIp = details.address;
      relayPort = details.tunnel.openvpn.port;
      relayProtocol = details.tunnel.openvpn.protocol;
    }

    return (
      <View style={styles.connect}>
        <View style={styles.map}>
          <Map style={{ width: '100%', height: '100%' }} {...this._getMapProps()} />
        </View>
        <View style={styles.container}>
          {/* show spinner when connecting */}
          {tunnelState === 'connecting' ? (
            <View style={styles.status_icon}>
              <Img source="icon-spinner" height={60} width={60} alt="" />
            </View>
          ) : null}

          <TunnelControl
            tunnelState={this.props.connection.status.state}
            selectedRelayName={this.props.selectedRelayName}
            city={this.props.connection.city}
            country={this.props.connection.country}
            hostname={this.props.connection.hostname}
            relayIp={relayIp}
            relayPort={relayPort}
            relayProtocol={relayProtocol}
            outIpv4={null}
            outIpv6={null}
            onConnect={this.props.onConnect}
            onDisconnect={this.props.onDisconnect}
            onSelectLocation={this.props.onSelectLocation}
          />

          <NotificationArea
            style={styles.notification_area}
            tunnelState={this.props.connection.status}
            version={this.props.version}
            openExternalLink={this.props.onExternalLink}
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

type TunnelControlProps = {
  tunnelState: TunnelState,
  selectedRelayName: string,
  city: ?string,
  country: ?string,
  hostname: ?string,
  relayIp: ?string,
  relayPort: ?number,
  relayProtocol: ?string,
  outIpv4: ?string,
  outIpv6: ?string,
  onConnect: () => void,
  onDisconnect: () => void,
  onSelectLocation: () => void,
};

type TunnelControlState = {
  showConnectionInfo: boolean,
};

class TunnelControl extends Component<TunnelControlProps, TunnelControlState> {
  state = {
    showConnectionInfo: false,
  };

  render() {
    const Location = ({ children }) => <View style={styles.status_location}>{children}</View>;
    const City = () => <Text style={styles.status_location_text}>{this.props.city}</Text>;
    const Country = () => <Text style={styles.status_location_text}>{this.props.country}</Text>;
    const Hostname = () => <Text style={styles.status_hostname}>{this.props.hostname || ''}</Text>;

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
      <AppButton.GreenButton onPress={this.props.onConnect}>
        {'Secure my connection'}
      </AppButton.GreenButton>
    );

    const Disconnect = () => (
      <AppButton.RedTransparentButton onPress={this.props.onDisconnect}>
        {'Disconnect'}
      </AppButton.RedTransparentButton>
    );

    const Cancel = () => (
      <AppButton.RedTransparentButton onPress={this.props.onDisconnect}>
        {'Cancel'}
      </AppButton.RedTransparentButton>
    );

    const Secured = ({ displayStyle }) => (
      <SecuredLabel style={styles.status_security} displayStyle={displayStyle} />
    );
    const Footer = ({ children }) => <View style={styles.footer}>{children}</View>;

    const connectionInfoProps = {
      inAddress: {
        ip: this.props.relayIp,
        port: this.props.relayPort,
        protocol: this.props.relayProtocol,
      },
      outAddress: {
        ipv4: this.props.outIpv4,
        ipv6: this.props.outIpv6,
      },
      startExpanded: this.state.showConnectionInfo,
      onToggle: (expanded) => {
        this.setState({ showConnectionInfo: expanded });
      },
    };

    switch (this.props.tunnelState) {
      case 'connecting':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.securing} />
              <Location>
                <City />
                <Country />
              </Location>
              <Hostname />
              <ConnectionInfo {...connectionInfoProps} />
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
              <Secured displayStyle={SecuredDisplayStyle.secured} />
              <Location>
                <City />
                <Country />
              </Location>
              <Hostname />
              <ConnectionInfo {...connectionInfoProps} />
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
              <Secured displayStyle={SecuredDisplayStyle.blocked} />
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
              <Secured displayStyle={SecuredDisplayStyle.secured} />
              <Location>
                <Country />
              </Location>
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
              <Secured displayStyle={SecuredDisplayStyle.unsecured} />
              <Location>
                <Country />
              </Location>
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

type ContainerProps = {
  children?: Types.ReactNode,
};

class Wrapper extends Component<ContainerProps> {
  render() {
    return <View style={styles.tunnel_control}>{this.props.children}</View>;
  }
}

class Body extends Component<ContainerProps> {
  render() {
    return <View style={styles.body}>{this.props.children}</View>;
  }
}
