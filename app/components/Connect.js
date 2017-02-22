import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { servers, ConnectionState } from '../constants';

export default class Connect extends Component {
  
  static propTypes = {
    settings: PropTypes.object.isRequired,
    onConnect: PropTypes.func.isRequired,
    onDisconnect: PropTypes.func.isRequired
  };

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

  serverName(key) {
    switch(key) {
    case 'fastest': return 'Fastest';
    case 'nearest': return 'Nearest';
    default: return (servers[key] || {}).name;
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

  render() {
    const preferredServer = this.props.settings.preferredServer;
    const serverName = this.serverName(preferredServer);
    const isConnecting = this.props.connect.status === ConnectionState.connecting;

    return (
      <Layout>
        <Header style={ this.headerStyle() } showSettings={ true } onSettings={ ::this.onSettings } />
        <Container>
          <div className="connect">
            <div className="connect__map"></div>
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
                <div className="connect__status-location">Gothenburg<br/>Sweden</div>
                <div className="connect__status-ipaddress">193.138.219.245</div>
              </div>

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

                      <div className="connect__server-name">{ serverName }</div>

                    </div>
                  </div>
                </div>

                <div className="connect__row">
                  <button className="connect__secure-button" onClick={ ::this.onConnect }>Secure my connection</button>
                </div>

              </div>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
