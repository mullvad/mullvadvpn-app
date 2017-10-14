// @flow
import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';

export type SupportProps = {
  onClose: () => void;
};

export default class Support extends Component {
  props: SupportProps;

  render() {
    return (
      <Layout>
        <Header hidden={ true } style={ 'defaultDark' } />
        <Container>
          <div className="support">
            <div className="support__close" onClick={ this.props.onClose }>
              <img className="support__close-icon" src="./assets/images/icon-back.svg" />
              <span className="support__close-title">Settings</span>
            </div>
            <div className="support__container">

              <div className="support__header">
                <h2 className="support__title">Support</h2>
              </div>

              <div className="support__content">
                <div className="support__main">
                </div>
              </div>

            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
