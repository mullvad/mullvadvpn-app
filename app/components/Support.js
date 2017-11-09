// @flow
import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';

export type SupportReport = {
  email: string,
  message: string,
  savedReport: ?string,
};

export type SupportState = {
  email: string,
  message: string,
  savedReport: ?string,
  sendState: 'INITIAL' | 'LOADING' | 'SUCCESS' | 'FAILED',
};
export type SupportProps = {
  onClose: () => void;
  onViewLog: (string) => void;
  onCollectLog: () => Promise<string>;
  onSend: (email: string, message: string, savedReport: string) => void;
};

export default class Support extends Component {
  props: SupportProps;
  state: SupportState = {
    email: '',
    message: '',
    savedReport: null,
    sendState: 'INITIAL',
  }

  validate() {
    return this.state.message.trim().length > 0;
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
    this.setState({ message: input.value });
  }

  onViewLog = () => {

    this._getLog()
      .then((path) => {
        this.props.onViewLog(path);
      });
  }

  _getLog() {
    const { savedReport } = this.state;
    return savedReport ?
      Promise.resolve(savedReport) :
      this.props.onCollectLog()
        .then( path => {
          return new Promise(resolve => this.setState({ savedReport: path }, () => resolve(path)));
        });
  }

  onSend = () => {
    this.setState({
      sendState: 'LOADING',
    }, () => {
      this._getLog()
        .then((path) => {
          return this.props.onSend(this.state.email, this.state.message, path);
        })
        .then( () => {
          this.setState({
            sendState: 'SUCCESS',
          });
        })
        .catch( () => {
          this.setState({
            sendState: 'FAILED',
          });
        });
    });
  }

  render() {

    const header = <div className="support__header">
      <h2 className="support__title">Report a problem</h2>
      { this.state.sendState === 'INITIAL' && <div className="support__subtitle">
        { `To help you more effectively, your app's log file will be attached to this message.
                    Your data will remain secure and private, as it is encrypted & anonymised before sending.` }
      </div>
      }
    </div>;

    const content = this._renderContent();

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

              { header }

              { content }

            </div>
          </div>
        </Container>
      </Layout>
    );
  }

  _renderContent() {
    switch(this.state.sendState) {
    case 'INITIAL':
      return this._renderForm();
    case 'LOADING':
      return this._renderLoading();
    case 'SUCCESS':
      return this._renderSent();
    case 'FAILED':
      return this._renderFailed();
    default:
      return null;
    }
  }

  _renderForm() {
    return <div className="support__content">
      <div className="support__form">
        <div className="support__form-row">
          <input className="support__form-email"
            type="email"
            placeholder="Your email"
            value={ this.state.email }
            onChange={ this.onChangeEmail }
            autoFocus={ true } />
        </div>
        <div className="support__form-row support__form-row-message">
          <div className="support__form-message-scroll-wrap">
            <textarea className="support__form-message"
              placeholder="Describe your problem"
              value={ this.state.message }
              onChange={ this.onChangeDescription } />
          </div>
        </div>
        <div className="support__footer">
          <button type="button"
            className="support__form-view-logs button button--primary"
            onClick={ this.onViewLog }>
            <span className="button-label">View app logs</span>
            <ExternalLinkSVG className="button-icon button-icon--16" />
          </button>
          <button type="button"
            className="support__form-send button button--positive"
            disabled={ !this.validate() }
            onClick={ this.onSend }>Send</button>
        </div>
      </div>
    </div>;
  }

  _renderLoading() {
    return <div className="support__content">

      <div className="support__form">
        <div className="support__form-row">
          <div className="support__status-icon">
            <img src="./assets/images/icon-spinner.svg" alt="" />
          </div>
          <div className="support__status-security--secure">
            Secure Connection
          </div>
          <div className="support__send-status">
            <span>Sending...</span>
          </div>
        </div>
      </div>
    </div>;
  }

  _renderSent() {
    return <div className="support__content">
      <div className="support__form">
        <div className="support__form-row">
          <div className="support__status-icon">
            <img src="./assets/images/icon-success.svg" alt="" />
          </div>
          <div className="support__status-security--secure">
            Secure Connection
          </div>
          <div className="support__send-status">
            <span>Sent</span>
          </div>
          <div className="support__subtitle">
            Thanks! We will look into this. If needed we will contact you on {'\u00A0'}
            <div className="support__sent-email">{ this.state.email }</div>
          </div>
        </div>
      </div>
    </div>;
  }

  _renderFailed() {
    return <div className="support__content">
      <div className="support__form">
        <div className="support__form-row">
          <div className="support__status-icon">
            <img src="./assets/images/icon-fail.svg" alt="" />
          </div>
          <div className="support__status-security--secure">
            Secure Connection
          </div>
          <div className="support__send-status">
            <span>Failed to send</span>
          </div>
        </div>
      </div>
      <div className="support__footer">
        <button type="button"
          className="support__form-view-logs button button--primary"
          onClick={ () => this.setState({ sendState: 'INITIAL' }) }>
          <span className="button-label">Edit message</span>
        </button>
        <button type="button"
          className="support__form-send button button--positive"
          onClick={ this.onSend }>Try again</button>
      </div>
    </div>;
  }
}
