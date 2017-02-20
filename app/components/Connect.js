import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';

export default class Connect extends Component {

  onSettings() {
    this.props.router.push('/settings');
  }

  openLocationPicker() {
    this.props.router.push('/select-location');
  }

  render() {
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
                    <div className="connect__server-country">{ this.props.settings.preferredServer }</div>
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
