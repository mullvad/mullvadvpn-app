import React, { Component, PropTypes } from 'react';
import { Layout, Container, Header } from './Layout';

export default class Settings extends Component {

  static propTypes = {
    logout: PropTypes.func.isRequired
  }

  onClose() {
    this.props.router.push('/connect');
  }

  render() {
    return (
      <Layout>
        <Header hidden={ true } />
        <Container>
          <div className="settings">
            <button className="settings__close" onClick={ ::this.onClose } />
          </div>
        </Container>
      </Layout>
    );
  }
}
