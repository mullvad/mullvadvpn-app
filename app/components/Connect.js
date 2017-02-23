import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import Leaflet from 'leaflet';
import cheapRuler from 'cheap-ruler';
import { Map, Marker, Popup, TileLayer } from 'react-leaflet';
import { Layout, Container, Header } from './Layout';
import { servers, ConnectionState } from '../constants';

export default class Connect extends Component {
  
  static propTypes = {
    settings: PropTypes.object.isRequired,
    onConnect: PropTypes.func.isRequired,
    onDisconnect: PropTypes.func.isRequired
  };

  constructor() {
    super();

    this.state = { 
      userLocation: [40.706213526877455, -74.0044641494751]
    };

    this._markerIcon = Leaflet.icon({
      iconUrl: './assets/images/icon-tick.svg',
      iconSize: [28, 20]
    });
  }

  onSettings() {
    this.props.router.push('/settings');
  }

  onSelectLocation() {
    this.props.router.push('/select-location');
  }

  onConnect() {
    const server = this.props.settings.preferredServer;
    this.props.onConnect(server);
  }

  onDisconnect() {
    this.props.onDisconnect();
  }

  serverInfo(key) {
    switch(key) {
    case 'fastest': 
      return {
        name: 'Fastest',
        city: 'New York',
        country: 'USA',
        location: [40.7127837, -74.0059413]
      };
    case 'nearest':
      return {
        name: 'Nearest',
        city: 'New York',
        country: 'USA',
        location: [40.7127837, -74.0059413]
      };
    default: return servers[key] || {};
    }
  }

  headerStyle() {
    const S = Header.Style;
    switch(this.props.connect.status) {
    case ConnectionState.disconnected: return S.error;
    case ConnectionState.connected: return S.success;
    default: return S.default;
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
    case ConnectionState.connected: return 'Secured connection';
    case ConnectionState.connecting: return 'Creating secure connection';
    default: return 'Unsecured connection';
    }
  }

  displayLocation() {
    if(this.props.connect.status === ConnectionState.disconnected) {
      return this.state.userLocation;
    }
    
    const preferredServer = this.props.settings.preferredServer;
    const serverInfo = this.serverInfo(preferredServer);

    return serverInfo.location;
  }

  getBounds(center) {
    const ruler = cheapRuler(center[0], 'meters');
    const bbox = ruler.bufferPoint(center, 100000);
    const p1 = Leaflet.latLng(bbox[0], bbox[1]);
    const p2 = Leaflet.latLng(bbox[2], bbox[3]);

    return Leaflet.latLngBounds(p1, p2);
  }

  render() {
    const tileURL = 'https://cartodb-basemaps-{s}.global.ssl.fastly.net/dark_all/{z}/{x}/{y}@2x.png';

    const preferredServer = this.props.settings.preferredServer;
    const serverInfo = this.serverInfo(preferredServer);
    const displayLocation = this.displayLocation();

    const isConnecting = this.props.connect.status === ConnectionState.connecting;
    const isConnected = this.props.connect.status === ConnectionState.connected;
    const isDisconnected = this.props.connect.status === ConnectionState.disconnected;

    return (
      <Layout>
        <Header style={ this.headerStyle() } showSettings={ true } onSettings={ ::this.onSettings } />
        <Container>
          <div className="connect">
            <div className="connect__map">
              <Map zoomControl={ false } 
                   bounds={ this.getBounds(displayLocation) }
                   boundsOptions={ { paddingBottomRight: [0, 150]} }
                   dragging={ false }
                   useFlyTo={ true }
                   animate={ true }
                   fadeAnimation={ false }
                   style={{ height: '100%', backgroundColor: 'black' }}>
                <TileLayer url={ tileURL } />
                <Marker position={ displayLocation } 
                        keyboard={ false } />
              </Map>
            </div>
            <div className="connect__container">

              <div className="connect__status">
                { /* show spinner when connecting */ }
                <If condition={ isConnecting }>
                  <Then>
                    <div className="connect__status-icon">
                      <img src="./assets/images/icon-spinner.svg" alt="" />
                    </div>
                  </Then>
                </If>
                
                <div className={ this.networkSecurityClass() }>{ this.networkSecurityMessage() }</div>
                <div className="connect__status-location">{ serverInfo.city }<br/>{ serverInfo.country }</div>
                <div className="connect__status-ipaddress">{ this.props.connect.clientIp }</div>
              </div>

              { /* footer when disconnected */ }
              <If condition={ isDisconnected }>
                <Then>
                  <div className="connect__footer">
                    <div className="connect__row">

                      <div className="connect__server" onClick={ ::this.onSelectLocation }>
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
                      <button className="connect__footer-button connect__footer-button--connect" onClick={ ::this.onConnect }>Secure my connection</button>
                    </div>
                  </div>
                </Then>
              </If>
              
              { /* footer when connecting */ }
              <If condition={ isConnecting }>
                <Then>
                  <div className="connect__footer">
                    <div className="connect__row">
                      <button className="connect__footer-button connect__footer-button--switch" onClick={ ::this.onSelectLocation }>Switch location</button>
                    </div>

                    <div className="connect__row">
                      <button className="connect__footer-button connect__footer-button--disconnect" onClick={ ::this.onDisconnect }>Cancel</button>
                    </div>
                  </div>
                </Then>
              </If>

              { /* footer when connected */ }
              <If condition={ isConnected }>
                <Then>
                  <div className="connect__footer">
                    <div className="connect__row">
                      <button className="connect__footer-button connect__footer-button--switch" onClick={ ::this.onSelectLocation }>Switch location</button>
                    </div>

                    <div className="connect__row">
                      <button className="connect__footer-button connect__footer-button--disconnect" onClick={ ::this.onDisconnect }>Disconnect</button>
                    </div>
                  </div>
                </Then>
              </If>

            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
