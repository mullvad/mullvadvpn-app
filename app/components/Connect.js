// @flow
import moment from 'moment';
import React, { Component } from 'react';
import { If, Then, Else } from 'react-if';
import ReactMapboxGl, { Marker } from 'react-mapbox-gl';
import cheapRuler from 'cheap-ruler';
import { Layout, Container, Header } from './Layout';
import { mapbox as mapboxConfig } from '../config';
import { BackendError } from '../lib/backend';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';

import type { Coordinate2d } from '../types';
import type { ServerInfo } from '../lib/backend';
import type { HeaderBarStyle } from './HeaderBar';
import type { AccountReduxState } from '../redux/account/reducers';
import type { ConnectionReduxState } from '../redux/connection/reducers';
import type { SettingsReduxState } from '../redux/settings/reducers';

type DisplayLocation = {
  location: Coordinate2d;
  country: ?string;
  city: ?string;
};

export type ConnectProps = {
  account: AccountReduxState,
  connection: ConnectionReduxState,
  settings: SettingsReduxState,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: (address: string) => void,
  onCopyIP: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
  getServerInfo: (identifier: string) => ?ServerInfo
};

export default class Connect extends Component {
  props: ConnectProps;
  state = {
    isFirstPass: true,
    showCopyIPMessage: false
  };

  _copyTimer: ?number;

  componentDidMount() {
    this.setState({ isFirstPass: false });
  }

  componentWillUnmount() {
    if(this._copyTimer) {
      clearTimeout(this._copyTimer);
      this._copyTimer = null;
    }

    this.setState({
      isFirstPass: true,
      showCopyIPMessage: false
    });
  }

  render(): React.Element<*> {
    const error = this.displayError();
    const child = error ? this.renderError(error) : this.renderMap();

    return (
      <Layout>
        <Header style={ this.headerStyle() } showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
          { child }
        </Container>
      </Layout>
    );
  }

  renderError(error: BackendError): React.Element<*> {
    return (
      <div className="connect">
        <div className="connect__status">
          <div className="connect__status-icon">
            <img src="./assets/images/icon-fail.svg" alt="" />
          </div>
          <div className="connect__error-title">
            { error.title }
          </div>
          <div className="connect__error-message">
            { error.message }
          </div>
          <If condition={ error.type === 'NO_CREDIT' }>
            <Then>
              <div>
                <button className="button button--positive" onClick={ this.onExternalLink.bind(this, 'purchase') }>
                  <span className="button-label">Buy more time</span>
                  <ExternalLinkSVG className="button-icon button-icon--16" />
                </button>
              </div>
            </Then>
          </If>
        </div>
      </div>
    );
  }

  renderMap(): React.Element<*> {
    const preferredServer = this.props.settings.preferredServer;
    const serverInfo = this.props.getServerInfo(preferredServer);
    if(!serverInfo) {
      throw new Error('Server info cannot be null.');
    }

    let isConnecting = false;
    let isConnected = false;
    let isDisconnected = false;
    switch(this.props.connection.status) {
    case 'connecting': isConnecting = true; break;
    case 'connected': isConnected = true; break;
    case 'disconnected': isDisconnected = true; break;
    }

    const altitude = (isConnecting ? 300 : 100) * 1000;
    const displayLocation = this.displayLocation();
    const mapBounds = this.calculateMapBounds(displayLocation.location, altitude);
    const mapBoundsOptions = { offset: [0, -113], animate: !this.state.isFirstPass };
    const accountLocation = this.convertToMapCoordinate(this.props.account.location || [0, 0]);
    const serverLocation = this.convertToMapCoordinate(serverInfo.location);

    const map = process.platform === 'darwin'
      ? <ReactMapboxGl
        style={ mapboxConfig.styleURL }
        accessToken={ mapboxConfig.accessToken }
        containerStyle={{ height: '100%' }}
        interactive={ false }
        fitBounds={ mapBounds }
        fitBoundsOptions={ mapBoundsOptions }>
        <If condition={ isConnected }>
          <Then>
            <Marker coordinates={ serverLocation } offset={ [0, -10] }>
              <img src='./assets/images/location-marker-secure.svg' />
            </Marker>
          </Then>
        </If>
        <If condition={ !isConnected }>
          <Then>
            <Marker coordinates={ accountLocation } offset={ [0, -10] }>
              <img src='./assets/images/location-marker-unsecure.svg' />
            </Marker>
          </Then>
        </If>
      </ReactMapboxGl>
      : undefined;

    return (
      <div className="connect">
        <div className="connect__map">
          { map }
        </div>
        <div className="connect__container">

          <div className="connect__status">
            { /* show spinner when connecting */ }
            <div className={ this.spinnerClass() }>
              <img src="./assets/images/icon-spinner.svg" alt="" />
            </div>

            <div className={ this.networkSecurityClass() }>{ this.networkSecurityMessage() }</div>

            { /*
              **********************************
              Begin: Location block
              **********************************
            */ }

            { /* location when connecting */ }
            <If condition={ isConnecting }>
              <Then>
                <div className="connect__status-location">

                  <If condition={ preferredServer === 'fastest' }>
                    <Then>
                      <span>
                        <img className="connect__status-location-icon" src="./assets/images/icon-fastest.svg" />
                        { 'Fastest' }
                      </span>
                    </Then>
                  </If>

                  <If condition={ preferredServer === 'nearest' }>
                    <Then>
                      <span>
                        <img className="connect__status-location-icon" src="./assets/images/icon-nearest.svg" />
                        { 'Nearest' }
                      </span>
                    </Then>
                  </If>

                  { /* silly but react-if does not have ElseIf */ }
                  <If condition={ preferredServer !== 'fastest' && preferredServer !== 'nearest' }>
                    <Then>
                      <span>{ displayLocation.country }</span>
                    </Then>
                  </If>

                </div>
              </Then>
            </If>

            { /* location when connected */ }
            <If condition={ isConnected }>
              <Then>
                <div className="connect__status-location">
                  { displayLocation.city }<br/>{ displayLocation.country }
                </div>
              </Then>
            </If>

            { /* location when disconnected */ }
            <If condition={ isDisconnected }>
              <Then>
                <div className="connect__status-location">
                  { displayLocation.country }
                </div>
              </Then>
            </If>

            { /*
              **********************************
              End: Location block
              **********************************
            */ }

            <div className={ this.ipAddressClass() } onClick={ this.onIPAddressClick.bind(this) }>
              <If condition={ this.state.showCopyIPMessage }>
                <Then><span>{ 'IP copied to clipboard!' }</span></Then>
                <Else><span>{ this.props.connection.clientIp }</span></Else>
              </If>
            </div>
          </div>


          { /*
            **********************************
            Begin: Footer block
            **********************************
          */ }

          { /* footer when disconnected */ }
          <If condition={ isDisconnected }>
            <Then>
              <div className="connect__footer">
                <div className="connect__row">

                  <div className="connect__server" onClick={ this.props.onSelectLocation }>
                    <div className="connect__server-label">Connect to</div>
                    <div className="connect__server-value">

                      <If condition={ preferredServer === 'fastest' }>
                        <Then>
                          <img className="connect__server-icon" src="./assets/images/icon-fastest.svg" />
                        </Then>
                      </If>

                      <If condition={ preferredServer === 'nearest' }>
                        <Then>
                          <img className="connect__server-icon" src="./assets/images/icon-nearest.svg" />
                        </Then>
                      </If>

                      <div className="connect__server-name">{ serverInfo.name }</div>

                    </div>
                  </div>
                </div>

                <div className="connect__row">
                  <button className="button button--positive" onClick={ this.onConnect.bind(this) }>Secure my connection</button>
                </div>
              </div>
            </Then>
          </If>

          { /* footer when connecting */ }
          <If condition={ isConnecting }>
            <Then>
              <div className="connect__footer">
                <div className="connect__row">
                  <button className="button button--neutral button--blur" onClick={ this.props.onSelectLocation }>Switch location</button>
                </div>

                <div className="connect__row">
                  <button className="button button--negative-light button--blur" onClick={ this.props.onDisconnect }>Cancel</button>
                </div>
              </div>
            </Then>
          </If>

          { /* footer when connected */ }
          <If condition={ isConnected }>
            <Then>
              <div className="connect__footer">
                <div className="connect__row">
                  <button className="button button--neutral button--blur" onClick={ this.props.onSelectLocation }>Switch location</button>
                </div>

                <div className="connect__row">
                  <button className="button button--negative-light button--blur" onClick={ this.props.onDisconnect }>Disconnect</button>
                </div>
              </div>
            </Then>
          </If>

          { /*
            **********************************
            End: Footer block
            **********************************
          */ }

        </div>
      </div>
    );
  }

  // Handlers

  onConnect() {
    const { preferredServer } = this.props.settings;
    const serverInfo = this.props.getServerInfo(preferredServer);
    if(serverInfo) {
      this.props.onConnect(serverInfo.address);
    }
  }

  onExternalLink(type: string) {
    this.props.onExternalLink(type);
  }

  onIPAddressClick() {
    this._copyTimer && clearTimeout(this._copyTimer);
    this._copyTimer = setTimeout(() => this.setState({ showCopyIPMessage: false }), 3000);
    this.setState({ showCopyIPMessage: true });
    this.props.onCopyIP();
  }

  // Private

  headerStyle(): HeaderBarStyle {
    switch(this.props.connection.status) {
    case 'connecting':
    case 'disconnected':
      return 'error';
    case 'connected':
      return 'success';
    }
    throw new Error('Invalid ConnectionState');
  }

  networkSecurityClass(): string {
    let classes = ['connect__status-security'];
    if(this.props.connection.status === 'connected') {
      classes.push('connect__status-security--secure');
    } else if(this.props.connection.status === 'disconnected') {
      classes.push('connect__status-security--unsecured');
    }

    return classes.join(' ');
  }

  networkSecurityMessage(): string {
    switch(this.props.connection.status) {
    case 'connected': return 'Secure connection';
    case 'connecting': return 'Creating secure connection';
    default: return 'Unsecured connection';
    }
  }

  spinnerClass(): string {
    var classes = ['connect__status-icon'];
    if(this.props.connection.status !== 'connecting') {
      classes.push('connect__status-icon--hidden');
    }
    return classes.join(' ');
  }

  ipAddressClass(): string {
    var classes = ['connect__status-ipaddress'];
    if(this.props.connection.status === 'connecting') {
      classes.push('connect__status-ipaddress--invisible');
    }
    return classes.join(' ');
  }

  displayLocation(): DisplayLocation {
    // return user location when disconnected
    if(this.props.connection.status === 'disconnected') {
      let { location, country, city } = this.props.account;
      return {
        location: location || [0, 0],
        country, city
      };
    } else { // otherwise server location
      const preferredServer = this.props.settings.preferredServer;
      const serverInfo = this.props.getServerInfo(preferredServer);
      if(serverInfo) {
        const { location, country, city } = serverInfo;
        return { location, country, city };
      }
      throw new Error('Server location is not available.');
    }
  }

  displayError(): ?BackendError {
    // Offline?
    if(!this.props.connection.isOnline) {
      return new BackendError('NO_INTERNET');
    }

    // No credit?
    const { paidUntil } = this.props.account;
    if(paidUntil && moment(paidUntil).isSameOrBefore(moment())) {
      return new BackendError('NO_CREDIT');
    }

    return null;
  }

  // Geo helpers

  calculateMapBounds(center: Coordinate2d, altitude: number): [Coordinate2d, Coordinate2d] {
    const bounds = cheapRuler(center[0], 'meters').bufferPoint(center, altitude);
    // convert [lat,lng] bounds to [lng,lat]
    return [ [bounds[1], bounds[0]], [bounds[3], bounds[2]] ];
  }

  convertToMapCoordinate(pos: Coordinate2d): Coordinate2d {
    // convert [lat,lng] bounds to [lng,lat]
    return [pos[1], pos[0]];
  }
}
