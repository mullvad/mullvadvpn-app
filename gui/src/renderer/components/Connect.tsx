import * as React from 'react';
import styled from 'styled-components';
import { hasExpired } from '../../shared/account-expiry';
import NotificationArea from '../components/NotificationArea';
import { AuthFailureKind, parseAuthFailure } from '../../shared/auth-failure';
import { LoginState } from '../redux/account/reducers';
import { IConnectionReduxState } from '../redux/connection/reducers';
import { calculateHeaderBarStyle, DefaultHeaderBar } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import Map, { MarkerStyle, ZoomLevel } from './Map';
import { ModalContainer } from './Modal';
import TunnelControl from './TunnelControl';

interface IProps {
  connection: IConnectionReduxState;
  loginState: LoginState;
  accountExpiry?: string;
  blockWhenDisconnected: boolean;
  selectedRelayName: string;
  onSelectLocation: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  onReconnect: () => void;
}

type MarkerOrSpinner = 'marker' | 'spinner';

const StyledMap = styled(Map)({
  position: 'absolute',
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  zIndex: 0,
});

const StyledContainer = styled(Container)({
  position: 'relative',
});

const Content = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  position: 'relative', // need this for z-index to work to cover the map
  zIndex: 1,
});

const StatusIcon = styled(ImageView)({
  position: 'absolute',
  alignSelf: 'center',
  marginTop: 94,
});

const StyledNotificationArea = styled(NotificationArea)({
  position: 'absolute',
  left: 0,
  top: 0,
  right: 0,
});

const StyledMain = styled.main({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

interface IState {
  isAccountExpired: boolean;
}

export default class Connect extends React.Component<IProps, IState> {
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
          <DefaultHeaderBar barStyle={calculateHeaderBarStyle(this.props.connection.status)} />
          <StyledContainer>{this.renderMap()}</StyledContainer>
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
      return hasExpired(this.props.accountExpiry);
    }

    // Do not assume that the account hasn't expired if the expiry is not available at the moment
    // instead return the last known state.
    return prevAccountExpired;
  }

  private renderMap() {
    return (
      <>
        <StyledMap {...this.getMapProps()} />
        <Content>
          <StyledNotificationArea />

          <StyledMain>
            {/* show spinner when connecting */}
            {this.showMarkerOrSpinner() === 'spinner' ? (
              <StatusIcon source="icon-spinner" height={60} width={60} />
            ) : null}

            <TunnelControl
              tunnelState={this.props.connection.status}
              blockWhenDisconnected={this.props.blockWhenDisconnected}
              selectedRelayName={this.props.selectedRelayName}
              city={this.props.connection.city}
              country={this.props.connection.country}
              onConnect={this.props.onConnect}
              onDisconnect={this.props.onDisconnect}
              onReconnect={this.props.onReconnect}
              onSelectLocation={this.props.onSelectLocation}
            />
          </StyledMain>
        </Content>
      </>
    );
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
        return !status.details.blockFailure ? MarkerStyle.secure : MarkerStyle.unsecure;
      case 'disconnected':
        return MarkerStyle.unsecure;
      case 'disconnecting':
        switch (status.details) {
          case 'block':
          case 'reconnect':
            return MarkerStyle.secure;
          case 'nothing':
            return MarkerStyle.unsecure;
        }
    }
  }

  private showMarkerOrSpinner(): MarkerOrSpinner {
    const status = this.props.connection.status;

    return status.state === 'connecting' || status.state === 'disconnecting' ? 'spinner' : 'marker';
  }
}
