// @flow
import React from 'react';
import { Component, Text, View, TextInput } from 'reactxp';
import { Button, AppButton } from './styled';
import { Layout, Container } from './Layout';
import styles from './SupportStyles';
import Img from './Img';

import type { AccountReduxState } from '../redux/account/reducers';

export type SupportReport = {
  email: string,
  message: string,
  savedReport: ?string,
};

export type SupportState = {
  email: string,
  message: string,
  savedReport: ?string,
  sendState: 'INITIAL' | 'CONFIRM_NO_EMAIL' | 'LOADING' | 'SUCCESS' | 'FAILED',
};
export type SupportProps = {
  account: AccountReduxState,
  onClose: () => void;
  onViewLog: (string) => void;
  onCollectLog: (Array<string>) => Promise<string>;
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

  onChangeEmail = (email: string) => {
    this.setState({ email: email });
  }

  onChangeDescription = (description: string) => {
    this.setState({ message: description });
  }

  onViewLog = () => {

    this._getLog()
      .then((path) => {
        this.props.onViewLog(path);
      });
  }

  _getLog() {
    const toRedact = [];
    if (this.props.account.accountToken) {
      toRedact.push(this.props.account.accountToken.toString());
    }

    const { savedReport } = this.state;
    return savedReport ?
      Promise.resolve(savedReport) :
      this.props.onCollectLog(toRedact)
        .then( path => {
          return new Promise(resolve => this.setState({ savedReport: path }, () => resolve(path)));
        });
  }

  onSend = () => {
    if (this.state.sendState === 'INITIAL' && this.state.email.length === 0) {
      this.setState({
        sendState: 'CONFIRM_NO_EMAIL',
      });
    } else {
      this._sendProblemReport();
    }
  }

  _sendProblemReport() {
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

    const { sendState } = this.state;
    const header = <View style={styles.support__header}>
      <Text style={styles.support__title}>Report a problem</Text>
      { (sendState === 'INITIAL' || sendState === 'CONFIRM_NO_EMAIL') && <Text style={styles.support__subtitle}>
        { 'To help you more effectively, your app\'s log file will be attached to this message. Your data will remain secure and private, as it is anonymised before being sent over an encrypted channel.' }
      </Text>
      }
    </View>;

    const content = this._renderContent();

    return (
      <Layout>
        <Container>
          <View style={styles.support}>
            <Button style={styles.support__close} onPress={ this.props.onClose } testName="support__close">
              <Img style={styles.support__close_icon} source="icon-back" />
              <Text style={styles.support__close_title}>Settings</Text>
            </Button>
            <View style={styles.support__container}>

              { header }

              { content }

            </View>
          </View>
        </Container>
      </Layout>
    );
  }

  _renderContent() {
    switch(this.state.sendState) {
    case 'INITIAL':
    case 'CONFIRM_NO_EMAIL':
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

    return <View style={styles.support__content}>
      <View style={styles.support__form}>
        <View style={styles.support__form_row}>
          <TextInput style={styles.support__form_email}
            placeholder="Your email"
            defaultValue={ this.state.email }
            onChangeText={ this.onChangeEmail }
            keyboardType="email-address"
            autoFocus={ true } />
        </View>
        <View style={styles.support__form_row_message}>
          <View style={styles.support__form_message_scroll_wrap}>
            <TextInput style={styles.support__form_message}
              placeholder="Describe your problem"
              defaultValue={ this.state.message }
              multiline={ true }
              onChangeText={ this.onChangeDescription }
              testName="support__form_message"/>
          </View>
        </View>
        <View style={styles.support__footer}>
          {
            this.state.sendState === 'CONFIRM_NO_EMAIL'
              ? this._renderNoEmailWarning()
              : this._renderActionButtons()
          }
        </View>
      </View>
    </View>;
  }

  _renderNoEmailWarning() {
    return <View>
      <Text style={styles.support__no_email_warning}>
      You are about to send the problem report without a way for us to get back to you. If you want an answer to your report you will have to enter an email address.
      </Text>
      <AppButton style={styles.support__form_send}
        hoverStyle={styles.support__form_send_hover}
        disabled={ !this.validate() }
        onPress={ this.onSend }
        testName='support__send_logs'
        text='Send anyway'/>
    </View>;
  }

  _renderActionButtons() {
    return [
      <AppButton key={1}
        onPress={ this.onViewLog }
        style={ styles.support__form_view_logs }
        testName='support__view_logs'
        text='View app logs'
        icon="icon-extLink"
        iconStyle={styles.support__open_icon}
        tintColor='currentColor'/>,
      <AppButton key={2}
        style={styles.support__form_send}
        hoverStyle={styles.support__form_send_hover}
        disabled={ !this.validate() }
        onPress={ this.onSend }
        testName='support__send_logs'
        text='Send'/>
    ];
  }

  _renderLoading() {
    return <View style={styles.support__content}>
      <View style={styles.support__form}>
        <View style={styles.support__form_row}>
          <View style={styles.support__status_icon}>
            <Img source="icon-spinner" alt="" />
          </View>
          <View style={styles.support__status_security__secure}>
            Secure Connection
          </View>
          <Text style={styles.support__send_status}>
            Sending...
          </Text>
        </View>
      </View>
    </View>;
  }

  _renderSent() {
    return <View style={styles.support__content}>
      <View style={styles.support__form}>
        <View style={styles.support__form_row}>
          <View style={styles.support__status_icon}>
            <Img source="icon-success" alt="" />
          </View>
          <Text style={styles.support__status_security__secure}>
            Secure Connection
          </Text>
          <Text style={styles.support__send_status}>
            Sent
          </Text>

          <Text style={styles.support__subtitle}>
            Thanks! We will look into this.
          </Text>
          { this.state.email.trim().length > 0  ?
            <Text style={styles.support__subtitle}>If needed we will contact you on {'\u00A0'}
              <Text style={styles.support__sent_email}>{ this.state.email }</Text>
            </Text>
            : null }
        </View>
      </View>
    </View>;
  }

  _renderFailed() {
    return <View style={styles.support__content}>
      <View style={styles.support__form}>
        <View style={styles.support__form_row}>
          <View style={styles.support__status_icon}>
            <Img source="icon-fail" alt="" />
          </View>
          <Text style={styles.support__status_security__secure}>
            Secure Connection
          </Text>
          <Text style={styles.support__send_status}>
            Failed to send
          </Text>
        </View>
      </View>
      <View style={styles.support__footer}>
        <AppButton style={styles.support__form_edit_logs}
          onPress={ () => this.setState({ sendState: 'INITIAL' }) }
          text='Edit message'/>
        <AppButton style={styles.support__form_send}
          hoverStyle={styles.support__form_send_hover}
          onPress={ this.onSend }
          testName='support__send_logs'
          text='Try again'/>
      </View>
    </View>;
  }
}
