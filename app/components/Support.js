// @flow
import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';

export type SupportReport = {
  email: string,
  description: string
};

export type SupportState = SupportReport;
export type SupportProps = {
  onClose: () => void;
  onViewLogs: () => void;
  onSend: (report: SupportReport) => void;
};

export default class Support extends Component {
  props: SupportProps;
  state: SupportState = {
    email: '',
    description: ''
  }

  validate() {
    return this.state.description.trim().length > 0;
  }

  onChangeEmail = (e: Event) => {
    const input = e.target;
    if(!(input instanceof HTMLInputElement)) {
      throw new Error('input must be an instance of HTMLInputElement');
    }
    this.setState({ email: input.value });
  }

  onChangeDescription = (e: Event) => {
    const input = e.target;
    if(!(input instanceof HTMLTextAreaElement)) {
      throw new Error('input must be an instance of HTMLTextAreaElement');
    }
    this.setState({ description: input.value });
  }

  onSend = () => {
    this.props.onSend(this.state);
  }

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
                <h2 className="support__title">Contact support</h2>
                <div className="support__subtitle">
                  { `To help you more effectively, your app's log file will be attached to this message.
                  Your data will remain secure and private, as it is encrypted and anonymised before sending.` }
                </div>
              </div>

              <div className="support__content">
                <div className="support__form">
                  <div className="support__form-row">
                    <input className="support__form-email"
                      type="email"
                      placeholder="Your email"
                      value={ this.state.email }
                      onChange={ this.onChangeEmail }
                      autoFocus={ true } />
                  </div>
                  <div className="support__form-row support__form-row--description">
                    <div className="support__form-description-scroll-wrap">
                      <textarea className="support__form-description"
                        placeholder="Describe your problem"
                        value={ this.state.description }
                        onChange={ this.onChangeDescription } />
                    </div>
                  </div>
                  <div className="support__footer">
                    <button type="button"
                      className="button button--primary"
                      onClick={ this.props.onViewLogs }>
                      <span className="support__form-view-logs button-label">View app logs</span>
                      <ExternalLinkSVG className="button-icon button-icon--16" />
                    </button>
                    <button type="button"
                      className="support__form-send button button--positive"
                      disabled={ !this.validate() }
                      onClick={ this.onSend }>Send</button>
                  </div>
                </div>
              </div>

            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
