import React, { Component, PropTypes } from 'react';
import { Layout, Container, Header } from './Layout';

export default class Connect extends Component {

  static propTypes = {
    logout: PropTypes.func.isRequired
  }

  onSettings() {
    this.props.router.push('/settings');
  }

  render() {
    return (
      <Layout>
        <Header showSettings={ true } onSettings={ ::this.onSettings } />
        <Container>
          <div className="connect">
            <button style={{ width: '100px', display: 'block', margin: '10px auto' }} onClick={ this.props.logout }>Log out</button>
          </div>
        </Container>
      </Layout>
    );
  }
}
