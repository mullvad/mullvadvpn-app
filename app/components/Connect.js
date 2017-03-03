import assert from 'assert';
import React, { Component, PropTypes } from 'react';
import { If, Then, Else } from 'react-if';
import ReactMapboxGl, { Marker } from 'react-mapbox-gl';
import cheapRuler from 'cheap-ruler';
import { Layout, Container, Header } from './Layout';
import { mapbox as mapboxConfig } from '../config';
import { ConnectionState } from '../enums';

export default class Connect extends Component {
  
  static propTypes = {
    settings: PropTypes.object.isRequired,
    onSettings: PropTypes.func.isRequired,
    onConnect: PropTypes.func.isRequired,
    onDisconnect: PropTypes.func.isRequired,
    getServerInfo: PropTypes.func.isRequired
  };

  constructor() {
    super();

    this.state = {
      isFirstPass: true,
      userLocation: {
        location: [28.358744, -14.053676],
        city: 'Corralejo',
        country: 'Spain'
      }
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
    const preferredServer = this.props.settings.preferredServer;
    const serverInfo = this.props.getServerInfo(preferredServer);

    const isConnecting = this.props.connect.status === ConnectionState.connecting;
    const isConnected = this.props.connect.status === ConnectionState.connected;
    const isDisconnected = this.props.connect.status === ConnectionState.disconnected;

    const altitude = (isConnecting ? 300 : 100) * 1000;

    const displayLocation = this.displayLocation();
    const bounds = this.getBounds(displayLocation.location, altitude);

    const userLocation = this.toLngLat(this.state.userLocation.location);
    const serverLocation = this.toLngLat(serverInfo.location);
    const mapBounds = this.toLngLatBounds(bounds);
    const mapBoundsOptions = { offset: [0, -113], animate: !this.state.isFirstPass };

    return (
      <Layout>
        <Header style={ this.headerStyle() } showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
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
                <If condition={ isConnecting }>
                  <Then>
                    <div className="connect__status-icon">
                      <img src="./assets/images/icon-spinner.svg" alt="" />
                    </div>
                  </Then>
                </If>
                
                <div className={ this.networkSecurityClass() }>{ this.networkSecurityMessage() }</div>

                <If condition={ isConnecting }>
                  <Then>
                    <div className="connect__status-location">
                    <If condition={ preferredServer === 'fastest' }>
                      <Then>
                        <img className="connect__status-location-icon" src="./assets/images/icon-fastest.svg" />
                      </Then>
                    </If>
                      
                    <If condition={ preferredServer === 'nearest' }>
                      <Then>
                        <img className="connect__status-location-icon" src="./assets/images/icon-nearest.svg" />
                      </Then>
                    </If>

                    { displayLocation.country }<br/><br/>
                    </div>
                  </Then>
                  <Else>
                    <div className="connect__status-location">
                      { displayLocation.city }<br/>{ displayLocation.country }
                    </div>
                  </Else>
                </If>

                <div className={ this.ipAddressClass() }>{ this.props.connect.clientIp }</div>
              </div>

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

            </div>
          </div>
        </Container>
      </Layout>
    );
  }

  // Handlers

  onConnect() {
    const server = this.props.settings.preferredServer;
    const serverInfo = this.props.getServerInfo(server);
    this.props.onConnect(serverInfo.address);
  }

  // Private

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
    case ConnectionState.connected: return 'Secure connection';
    case ConnectionState.connecting: return 'Creating secure connection';
    default: return 'Unsecured connection';
    }
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
      return this.state.userLocation;
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
    assert(pos.length === 2); 
    return [ pos[1], pos[0] ]; 
  }

  toLngLatBounds(bounds) {
    assert(bounds.length % 2 === 0);
    let result = [];
    for(let i = 0; i < bounds.length; i += 2) {
      result.push(bounds.slice(i, i + 2).reverse());
    }
    return result;
  }
}
