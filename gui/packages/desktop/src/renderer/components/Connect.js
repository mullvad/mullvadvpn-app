// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Text, View, Types } from 'reactxp';
import { Accordion, ClipboardLabel, SecuredLabel, SecuredDisplayStyle } from '@mullvad/components';
import { Layout, Container, Header } from './Layout';
import { SettingsBarButton, Brand } from './HeaderBar';
import BlockingInternetBanner, { BannerTitle, BannerSubtitle } from './BlockingInternetBanner';
import * as AppButton from './AppButton';
import Img from './Img';
import Map from './Map';
import styles from './ConnectStyles';
import { NoCreditError, NoInternetError } from '../errors';
import type { BlockReason, TunnelState, TunnelStateTransition } from '../lib/daemon-rpc';

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
            throw new Error(`Invalid action after disconnection: ${(status.details: empty)}`);
        }
      default:
        throw new Error(`Invalid connection status: ${(status.state: empty)}`);
    }
  }

  renderMap() {
    return (
      <View style={styles.connect}>
        <View style={styles.map}>
          <Map style={{ width: '100%', height: '100%' }} {...this._getMapProps()} />
        </View>
        <View style={styles.container}>
          <TunnelBanner tunnelState={this.props.connection.status} />

          {/* show spinner when connecting */}
          {this.props.connection.status.state === 'connecting' ? (
            <View style={styles.status_icon}>
              <Img source="icon-spinner" height={60} width={60} alt="" />
            </View>
          ) : null}

          <TunnelControl
            style={styles.tunnel_control}
            tunnelState={this.props.connection.status.state}
            selectedRelayName={this.props.selectedRelayName}
            city={this.props.connection.city}
            country={this.props.connection.country}
            hostname={this.props.connection.hostname}
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

type TunnelBannerProps = {
  tunnelState: TunnelStateTransition,
};

type TunnerBannerState = {
  visible: boolean,
  title: string,
  subtitle: string,
};

export class TunnelBanner extends Component<TunnelBannerProps, TunnerBannerState> {
  state = {
    visible: false,
    title: '',
    subtitle: '',
  };

  constructor(props: TunnelBannerProps) {
    super();
    this.state = this._deriveState(props.tunnelState);
  }

  componentDidUpdate(oldProps: TunnelBannerProps, _oldState: TunnerBannerState) {
    if (
      oldProps.tunnelState.state !== this.props.tunnelState.state ||
      oldProps.tunnelState.details !== this.props.tunnelState.details
    ) {
      const nextState = this._deriveState(this.props.tunnelState);
      this.setState(nextState);
    }
  }

  render() {
    return (
      <Accordion style={styles.blocking_container} height={this.state.visible ? 'auto' : 0}>
        <BlockingInternetBanner>
          <BannerTitle>{this.state.title}</BannerTitle>
          <BannerSubtitle>{this.state.subtitle}</BannerSubtitle>
        </BlockingInternetBanner>
      </Accordion>
    );
  }

  _deriveState(tunnelState: TunnelStateTransition) {
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
          ...this.state,
          visible: false,
        };
    }
  }
}

type TunnelControlProps = {
  tunnelState: TunnelState,
  selectedRelayName: string,
  city: ?string,
  country: ?string,
  hostname: ?string,
  onConnect: () => void,
  onDisconnect: () => void,
  onSelectLocation: () => void,
  style: Types.ViewStyleRuleSet,
};

export function TunnelControl(props: TunnelControlProps) {
  const Location = ({ children }) => <View style={styles.status_location}>{children}</View>;
  const City = () => <Text style={styles.status_location_text}>{props.city}</Text>;
  const Country = () => <Text style={styles.status_location_text}>{props.country}</Text>;
  const Hostname = () => <Text style={styles.status_hostname}>{props.hostname || ''}</Text>;

  const SwitchLocation = () => {
    return (
      <AppButton.TransparentButton
        style={styles.switch_location_button}
        onPress={props.onSelectLocation}>
        <AppButton.Label>{'Switch location'}</AppButton.Label>
      </AppButton.TransparentButton>
    );
  };

  const SelectedLocation = () => (
    <AppButton.TransparentButton
      style={styles.switch_location_button}
      onPress={props.onSelectLocation}>
      <AppButton.Label>{props.selectedRelayName}</AppButton.Label>
      <Img height={12} width={7} source="icon-chevron" />
    </AppButton.TransparentButton>
  );

  const Connect = () => (
    <AppButton.GreenButton onPress={props.onConnect}>
      {'Secure my connection'}
    </AppButton.GreenButton>
  );

  const Disconnect = () => (
    <AppButton.RedTransparentButton onPress={props.onDisconnect}>
      {'Disconnect'}
    </AppButton.RedTransparentButton>
  );

  const Cancel = () => (
    <AppButton.RedTransparentButton onPress={props.onDisconnect}>
      {'Cancel'}
    </AppButton.RedTransparentButton>
  );

  const Secured = ({ displayStyle }) => (
    <SecuredLabel style={styles.status_security} displayStyle={displayStyle} />
  );
  const Wrapper = ({ children }) => <View style={props.style}>{children}</View>;
  const Body = ({ children }) => <View style={styles.body}>{children}</View>;
  const Footer = ({ children }) => <View style={styles.footer}>{children}</View>;

  switch (props.tunnelState) {
    case 'connecting':
      return (
        <Wrapper>
          <Body>
            <Secured displayStyle={SecuredDisplayStyle.securing} />
            <Location>
              <City />
            </Location>
            <Hostname />
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
      throw new Error(`Unknown TunnelState: ${(props.tunnelState: empty)}`);
  }
}
