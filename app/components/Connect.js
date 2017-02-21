import React, { Component } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { servers } from '../constants';

export default class Connect extends Component {

  onSettings() {
    this.props.router.push('/settings');
  }

  openLocationPicker() {
    this.props.router.push('/select-location');
  }

  serverName(key) {
    switch(key) {
      case 'fastest': return 'Fastest';
      case 'nearest': return 'Nearest';
      default: return (servers[key] || {}).name
    }
  }

  render() {
    const preferredServer = this.props.settings.preferredServer;
    const serverName = this.serverName(preferredServer);
    
    return (
      <Layout>
        <Header showSettings={ true } onSettings={ ::this.onSettings } />
        <Container>
          <div className="connect">
            <div className="connect__map"></div>
            <div className="connect__container">
              <div className="connect__footer">
                
                <div className="connect__row">
                  <div className="connect__server" onClick={ ::this.openLocationPicker }>
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
                  <button className="connect__secure-button">Secure my connection</button>
                </div>

              </div>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
