import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';
import { servers } from '../constants';

export default class Connect extends Component {

  onSettings() {
    this.props.router.push('/settings');
  }

  openLocationPicker() {
    this.props.router.push('/select-location');
  }

  render() {
    let serverName;
    const preferredServer = this.props.settings.preferredServer;

    // special types of servers (Fastest, Nearest) 
    if(preferredServer === 'Fastest' || preferredServer === 'Nearest') {
      serverName = preferredServer;
    } else {
      serverName = (servers[preferredServer] || {}).name;
    }
    
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
                    <div className="connect__server-country">{ serverName }</div>
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
