import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';

export default class Connect extends Component {

  onSettings() {
    this.props.router.push('/settings');
  }

  render() {
    return (
      <Layout>
        <Header showSettings={ true } onSettings={ ::this.onSettings } />
        <Container>
          <div className="connect">
            <div className="map"></div>
            <div className="container">
              <div className="connect-pane">
                
                <div className="connect-pane__row">
                  <div className="connect-pane__server">
                    <div className="connect-pane__server-label">CONNECT TO</div>
                    <div className="connect-pane__server-country">USA</div>
                  </div>
                </div>

                <div className="connect-pane__row">
                  <button className="connect-pane__secure-button">Secure my connection</button>
                </div>

              </div>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
