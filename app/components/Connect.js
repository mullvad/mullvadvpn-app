import React, { Component, PropTypes } from 'react';
import { If, Then, Else } from 'react-if';
import cheapRuler from 'cheap-ruler';
import ReactMapboxGl, { Marker } from 'react-mapbox-gl';
import { Layout, Container, Header } from './Layout';
import { ConnectionState } from '../constants';

export default class Connect extends Component {
  
  static propTypes = {
    settings: PropTypes.object.isRequired,
    onConnect: PropTypes.func.isRequired,
    onDisconnect: PropTypes.func.isRequired,
    getServerInfo: PropTypes.func.isRequired
  };

  constructor() {
    super();

    this.state = { 
      userLocation: {
        location: [28.358744, -14.053676],
        city: 'Corralejo',
        country: 'Fuerteventura'
      }
    };
  }

  onSettings() {
    this.props.router.push('/settings');
  }

  onSelectLocation() {
    this.props.router.push('/select-location');
  }

  onConnect() {
    const server = this.props.settings.preferredServer;
    const serverInfo = this.props.getServerInfo(server);
    this.props.onConnect(serverInfo.address);
  }

  onDisconnect() {
    this.props.onDisconnect();
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

  getBounds(center, altitude) {
    const ruler = cheapRuler(center[0], 'meters');
    const bbox = ruler.bufferPoint(center, altitude);
    return [ bbox[1], bbox[0], bbox[3], bbox[2] ]; // <lng>, <lat>, <lng>, <lat>
  }

  componentWillMount() {
    const loc = this.displayLocation().location;

    // we need this to override default center
    // see: https://github.com/alex3165/react-mapbox-gl/issues/134
    this._initialLocation = [ loc[1], loc[0] ]; // <lng>, <lat>
  }

  componentWillUnmount() {
    this._initialLocation = null;
  }

  render() {
    const preferredServer = this.props.settings.preferredServer;
    const serverInfo = this.props.getServerInfo(preferredServer);

    const displayLocation = this.displayLocation(); // <lat>, <lng>
    const userLocation = this.state.userLocation.location; // <lat>, <lng>
    const serverLocation = serverInfo.location; // <lat>, <lng>

    const isConnecting = this.props.connect.status === ConnectionState.connecting;
    const isConnected = this.props.connect.status === ConnectionState.connected;
    const isDisconnected = this.props.connect.status === ConnectionState.disconnected;

    const altitude = (isConnecting ? 300 : 100) * 1000;

    const accessToken = 'pk.eyJ1IjoibWpob21lciIsImEiOiJjaXd3NmdmNHEwMGtvMnlvMGl3b3R5aGcwIn0.SqIPBcCP6-b9yjxCD32CNg';

    return (
      <Layout>
        <Header style={ this.headerStyle() } showSettings={ true } onSettings={ ::this.onSettings } />
        <Container>
          <div className="connect">
            <div className="connect__map">
              <ReactMapboxGl style="mapbox://styles/mjhomer/cizjoenga006f2smnm9z52u8e"
                  center={ this._initialLocation }
                  accessToken={ accessToken }
                  containerStyle={{ height: '100%' }} 
                  interactive={ false }
                  fitBounds={ this.getBounds(displayLocation.location, altitude) }
                  fitBoundsOptions={ {offset: [0, -100]} }>
                <If condition={ isConnected }>
                  <Then>
                    <Marker coordinates={ [ serverLocation[1], serverLocation[0] ] } offset={ [0, -10] }>
                      <img src='./assets/images/location-marker-secure.svg' />
                    </Marker>
                  </Then>
                </If>

                { /* user location marker */ }
                <If condition={ !isConnected }>
                  <Then>
                    <Marker coordinates={ [ userLocation[1], userLocation[0] ] } offset={ [0, -10] }>
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
                      <button className="button button--neutral button--blur" onClick={ ::this.onSelectLocation }>Switch location</button>
                    </div>

                    <div className="connect__row">
                      <button className="button button--negative-light button--blur" onClick={ ::this.onDisconnect }>Cancel</button>
                    </div>
                  </div>
                </Then>
              </If>

              { /* footer when connected */ }
              <If condition={ isConnected }>
                <Then>
                  <div className="connect__footer">
                    <div className="connect__row">
                      <button className="button button--neutral button--blur" onClick={ ::this.onSelectLocation }>Switch location</button>
                    </div>

                    <div className="connect__row">
                      <button className="button button--negative-light button--blur" onClick={ ::this.onDisconnect }>Disconnect</button>
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
