import React, { Component, PropTypes } from 'react';
import { Layout, Container, Header } from './Layout';

export default class Connect extends Component {

  static propTypes = {
    logout: PropTypes.func.isRequired
  }

  render() {
    return (
      <Layout>
        <Header />
        <Container>
          <div className="connect">
            <button style={{ width: '100px', display: 'block', margin: '10px auto' }} onClick={ this.props.logout }>Log out</button>
          </div>
        </Container>
      </Layout>
    );
  }
}
