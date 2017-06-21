import assert from 'assert';
import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { If, Then, Else } from 'react-if';
import ReactMapboxGl, { Marker } from 'react-mapbox-gl';
import cheapRuler from 'cheap-ruler';
import { Layout, Container, Header } from './Layout';
import { mapbox as mapboxConfig } from '../config';
import Backend from '../lib/backend';
import { ConnectionState } from '../enums';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';

import type HeaderBarStyle from './HeaderBar';

export default class Connect extends Component {

  static propTypes = {
    settings: PropTypes.object.isRequired,
    onSettings: PropTypes.func.isRequired,
    onConnect: PropTypes.func.isRequired,
    onCopyIP: PropTypes.func.isRequired,
    onDisconnect: PropTypes.func.isRequired,
    onExternalLink: PropTypes.func.isRequired,
    getServerInfo: PropTypes.func.isRequired
  };

  constructor() {
    super();

    // timer used along with `state.showCopyIPMessage`
    this._copyTimer = null;

    this.state = {
      isFirstPass: true,

      // this flag is used together with timer to display
      // a message that IP address has been copied to clipboard
      showCopyIPMessage: false
    };
  }

  // Component Lifecycle

  componentDidMount() {
    this.setState({ isFirstPass: false });
  }

  componentWillUnmount() {
    this.setState({ isFirstPass: true });
  }

  render() {
    let error = null;

    // check if user out of time
    // this is by far the simplest implementation
    // later on backend will notify us and disconnect VPN etc..
    if(moment(this.props.user.paidUntil).isSameOrBefore(moment())) {
      error = new Backend.Error(Backend.ErrorType.noCredit);
    }

    // Offline?
    if(this.props.connect.isOnline === false) {
      error = new Backend.Error(Backend.ErrorType.noInternetConnection);
    }

    return (
      <Layout>
        <Header style={ this.headerStyle() } showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
          <If condition={ error !== null }>
            <Then>{ () => this.renderError(error) }</Then>
            <Else>{ ::this.renderMap }</Else>
          </If>
        </Container>
      </Layout>
    );
  }

  renderError(error) {
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
          <If condition={ error.code === Backend.ErrorType.noCredit }>
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

  renderMap() {
    const preferredServer = this.props.settings.preferredServer;
    const serverInfo = this.props.getServerInfo(preferredServer);

    const isConnecting = this.props.connect.status === ConnectionState.connecting;
    const isConnected = this.props.connect.status === ConnectionState.connected;
    const isDisconnected = this.props.connect.status === ConnectionState.disconnected;

    const altitude = (isConnecting ? 300 : 100) * 1000;

    const displayLocation = this.displayLocation();
    const bounds = this.getBounds(displayLocation.location, altitude);

    const userLocation = this.toLngLat(this.props.user.location);
    const serverLocation = this.toLngLat(serverInfo.location);
    const mapBounds = this.toLngLatBounds(bounds);
    const mapBoundsOptions = { offset: [0, -113], animate: !this.state.isFirstPass };

    return (
      <div className="connect">
        <div className="connect__map">
          <ReactMapboxGl
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
                <Marker coordinates={ userLocation } offset={ [0, -10] }>
                  <img src='./assets/images/location-marker-unsecure.svg' />
                </Marker>
              </Then>
            </If>

          </ReactMapboxGl>
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

            <div className={ this.ipAddressClass() } onClick={ ::this.onIPAddressClick }>
              <If condition={ this.state.showCopyIPMessage }>
                <Then><span>{ 'IP copied to clipboard!' }</span></Then>
                <Else><span>{ this.props.connect.clientIp }</span></Else>
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
                  <button className="button button--positive" onClick={ ::this.onConnect }>Secure my connection</button>
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
    const server = this.props.settings.preferredServer;
    const serverInfo = this.props.getServerInfo(server);
    this.props.onConnect(serverInfo.address);
  }

  onExternalLink(type) {
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
    switch(this.props.connect.status) {
    case ConnectionState.connecting:
    case ConnectionState.disconnected:
      return 'error';
    case ConnectionState.connected:
      return 'success';
    }
  }

  networkSecurityClass() {
    let classes = ['connect__status-security'];
    if(this.props.connect.status === ConnectionState.connected) {
      classes.push('connect__status-security--secure');
    } else if(this.props.connect.status === ConnectionState.disconnected) {
      classes.push('connect__status-security--unsecured');
    }

    return classes.join(' ');
  }

  networkSecurityMessage() {
    switch(this.props.connect.status) {
    case ConnectionState.connected: return 'Secure connection';
    case ConnectionState.connecting: return 'Creating secure connection';
    default: return 'Unsecured connection';
    }
  }

  spinnerClass() {
    var classes = ['connect__status-icon'];
    if(this.props.connect.status !== ConnectionState.connecting) {
      classes.push('connect__status-icon--hidden');
    }
    return classes.join(' ');
  }

  ipAddressClass() {
    var classes = ['connect__status-ipaddress'];
    if(this.props.connect.status === ConnectionState.connecting) {
      classes.push('connect__status-ipaddress--invisible');
    }
    return classes.join(' ');
  }

  displayLocation() {
    if(this.props.connect.status === ConnectionState.disconnected) {
      const { location, country, city } = this.props.user;
      return { location, country, city };
    }

    const preferredServer = this.props.settings.preferredServer;
    return this.props.getServerInfo(preferredServer);
  }

  // Geo helpers

  getBounds(center, altitude) {
    const ruler = cheapRuler(center[0], 'meters');
    return ruler.bufferPoint(center, altitude);
  }

  toLngLat(pos) {
    assert(pos.length === 2, 'wrong number of coordinates in position');
    return [ pos[1], pos[0] ];
  }

  toLngLatBounds(bounds) {
    assert(bounds.length % 2 === 0, 'wrong number of sides in bounds');
    let result = [];
    for(let i = 0; i < bounds.length; i += 2) {
      result.push(bounds.slice(i, i + 2).reverse());
    }
    return result;
  }
}
