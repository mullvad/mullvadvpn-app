import * as React from 'react';
import { Component, Styles, View } from 'reactxp';
import AccountExpiry from '../../shared/account-expiry';
import ExpiredAccountErrorViewContainer from '../containers/ExpiredAccountErrorViewContainer';
import NotificationAreaContainer from '../containers/NotificationAreaContainer';
import { AuthFailureKind, parseAuthFailure } from '../lib/auth-failure';
import { LoginState } from '../redux/account/reducers';
import { IConnectionReduxState } from '../redux/connection/reducers';
import { Brand, HeaderBarStyle, SettingsBarButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Header, Layout } from './Layout';
import Map, { MarkerStyle, ZoomLevel } from './Map';
import { ModalContainer } from './Modal';
import TunnelControl from './TunnelControl';

interface IProps {
  connection: IConnectionReduxState;
  loginState: LoginState;
  accountExpiry?: AccountExpiry;
  selectedRelayName: string;
  onSettings: () => void;
  onSelectLocation: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  onReconnect: () => void;
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
      isAccountExpired: this.checkAccountExpired(false),
    };
  }

  public componentDidUpdate() {
    this.updateAccountExpired();
  }

  public render() {
    return (
      <ModalContainer>
        <Layout>
          <Header barStyle={this.headerBarStyle()}>
            <Brand />
            <SettingsBarButton onPress={this.props.onSettings} />
          </Header>
          <Container>
            {this.state.isAccountExpired ||
            (this.props.loginState.type === 'ok' &&
              this.props.loginState.method === 'new_account') ? (
              <ExpiredAccountErrorViewContainer />
            ) : (
              this.renderMap()
            )}
          </Container>
        </Layout>
      </ModalContainer>
    );
  }

  private updateAccountExpired() {
    const nextAccountExpired = this.checkAccountExpired(this.state.isAccountExpired);

    if (nextAccountExpired !== this.state.isAccountExpired) {
      this.setState({
        isAccountExpired: nextAccountExpired,
      });
    }
  }

  private checkAccountExpired(prevAccountExpired: boolean): boolean {
    const tunnelState = this.props.connection.status;

    // Blocked with auth failure / expired account
    if (
      tunnelState.state === 'error' &&
      tunnelState.details.cause.reason === 'auth_failed' &&
      parseAuthFailure(tunnelState.details.cause.reason).kind === AuthFailureKind.expiredAccount
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
            onReconnect={this.props.onReconnect}
            onSelectLocation={this.props.onSelectLocation}
          />

          <NotificationAreaContainer style={styles.notificationArea} />
        </View>
      </View>
    );
  }

  private headerBarStyle(): HeaderBarStyle {
    const { status } = this.props.connection;
    switch (status.state) {
      case 'disconnected':
        return HeaderBarStyle.error;
      case 'connecting':
      case 'connected':
        return HeaderBarStyle.success;
      case 'error':
        return status.details.isBlocking ? HeaderBarStyle.success : HeaderBarStyle.error;
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
        zoomLevel: state === 'connected' ? ZoomLevel.high : ZoomLevel.medium,
        // a magic offset to align marker with spinner
        offset: [0, 123],
      };
    } else {
      return {
        center: [0, 0],
        showMarker: false,
        markerStyle: MarkerStyle.unsecure,
        // show the world when user location is not known
        zoomLevel: ZoomLevel.low,
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
      case 'error':
        return status.details.isBlocking ? MarkerStyle.secure : MarkerStyle.unsecure;
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
