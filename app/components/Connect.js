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
          
          </div>
        </Container>
      </Layout>
    );
  }
}
