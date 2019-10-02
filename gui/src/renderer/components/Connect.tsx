import * as React from 'react';
import { Component, Styles, View } from 'reactxp';
import { links } from '../../config.json';
import NotificationAreaContainer from '../containers/NotificationAreaContainer';
import AccountExpiry from '../lib/account-expiry';
import { AuthFailureKind, parseAuthFailure } from '../lib/auth-failure';
import { IConnectionReduxState } from '../redux/connection/reducers';
import { IVersionReduxState } from '../redux/version/reducers';
import ExpiredAccountErrorView, { RecoveryAction } from './ExpiredAccountErrorView';
import { Brand, HeaderBarStyle, SettingsBarButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Header, Layout } from './Layout';
import Map, { MarkerStyle, ZoomLevel } from './Map';
import TunnelControl from './TunnelControl';

interface IProps {
  connection: IConnectionReduxState;
  version: IVersionReduxState;
  accountExpiry?: AccountExpiry;
  selectedRelayName: string;
  blockWhenDisconnected: boolean;
  onSettings: () => void;
  onSelectLocation: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  onExternalLink: (url: string) => Promise<void>;
  onExternalLinkWithAuth: (url: string) => Promise<void>;
}

type MarkerOrSpinner = 'marker' | 'spinner';

const styles = {
  connect: Styles.createViewStyle({
    flex: 1,
  }),
  map: Styles.createViewStyle({
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    // @ts-ignore
    zIndex: 0,
  }),
  body: Styles.createViewStyle({
    flex: 1,
    paddingTop: 0,
    paddingLeft: 24,
    paddingRight: 24,
    paddingBottom: 0,
    marginTop: 176,
  }),
  container: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'column',
    position: 'relative' /* need this for z-index to work to cover the map */,
    // @ts-ignore
    zIndex: 1,
  }),
  statusIcon: Styles.createViewStyle({
    position: 'absolute',
    alignSelf: 'center',
    width: 60,
    height: 60,
    marginTop: 94,
  }),
  notificationArea: Styles.createViewStyle({
    position: 'absolute',
    left: 0,
    top: 0,
    right: 0,
  }),
};

interface IState {
  isAccountExpired: boolean;
}

export default class Connect extends Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    this.state = {
      isAccountExpired: this.checkAccountExpired(props, false),
    };
  }

  public componentDidUpdate() {
    this.updateAccountExpired();
  }

  public render() {
    return (
      <Layout>
        <Header barStyle={this.headerBarStyle()}>
          <Brand />
          <SettingsBarButton onPress={this.props.onSettings} />
        </Header>
        <Container>
          {this.state.isAccountExpired ? this.renderExpiredAccountView() : this.renderMap()}
        </Container>
      </Layout>
    );
  }

  private updateAccountExpired() {
    const nextAccountExpired = this.checkAccountExpired(this.props, this.state.isAccountExpired);

    if (nextAccountExpired !== this.state.isAccountExpired) {
      this.setState({
        isAccountExpired: nextAccountExpired,
      });
    }
  }

  private checkAccountExpired(props: IProps, prevAccountExpired: boolean): boolean {
    const tunnelState = props.connection.status;

    // Blocked with auth failure / expired account
    if (
      tunnelState.state === 'blocked' &&
      tunnelState.details.reason === 'auth_failed' &&
      parseAuthFailure(tunnelState.details.details).kind === AuthFailureKind.expiredAccount
    ) {
      return true;
    }

    // Use the account expiry to deduce the account state
    if (this.props.accountExpiry) {
      return this.props.accountExpiry.hasExpired();
    }

    // Do not assume that the account hasn't expired if the expiry is not available at the moment
    // instead return the last known state.
    return prevAccountExpired;
  }

  private renderExpiredAccountView() {
    return (
      <ExpiredAccountErrorView
        blockWhenDisconnected={this.props.blockWhenDisconnected}
        isBlocked={this.props.connection.isBlocked}
        action={this.handleExpiredAccountRecovery}
      />
    );
  }

  private renderMap() {
    return (
      <View style={styles.connect}>
        <Map style={styles.map} {...this.getMapProps()} />
        <View style={styles.container}>
          {/* show spinner when connecting */}
          {this.showMarkerOrSpinner() === 'spinner' ? (
            <View style={styles.statusIcon}>
              <ImageView source="icon-spinner" height={60} width={60} />
            </View>
          ) : null}

          <TunnelControl
            tunnelState={this.props.connection.status}
            selectedRelayName={this.props.selectedRelayName}
            city={this.props.connection.city}
            country={this.props.connection.country}
            onConnect={this.props.onConnect}
            onDisconnect={this.props.onDisconnect}
            onSelectLocation={this.props.onSelectLocation}
          />

          <NotificationAreaContainer style={styles.notificationArea} />
        </View>
      </View>
    );
  }

  private handleExpiredAccountRecovery = async (recoveryAction: RecoveryAction) => {
    switch (recoveryAction) {
      case RecoveryAction.disableBlockedWhenDisconnected:
        break;

      case RecoveryAction.openBrowser:
        this.props.onExternalLink(links.purchase);
        break;

      case RecoveryAction.disconnectAndOpenBrowser:
        try {
          await this.props.onDisconnect();
          this.props.onExternalLink(links.purchase);
        } catch (error) {
          // no-op
        }
    }
  };

  private headerBarStyle(): HeaderBarStyle {
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

  private getMapProps(): Map['props'] {
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
        showMarker: this.showMarkerOrSpinner() === 'marker',
        markerStyle: this.getMarkerStyle(),
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

  private getMarkerStyle(): MarkerStyle {
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

  private showMarkerOrSpinner(): MarkerOrSpinner {
    const status = this.props.connection.status;

    return status.state === 'connecting' ||
      (status.state === 'disconnecting' && status.details === 'reconnect')
      ? 'spinner'
      : 'marker';
  }
}
