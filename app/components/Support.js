// @flow

import * as React from 'react';
import { Component, Text, View, TextInput } from 'reactxp';
import * as AppButton from './AppButton';
import { Layout, Container } from './Layout';
import NavigationBar, { BackBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle, HeaderSubTitle } from './SettingsHeader';
import styles from './SupportStyles';
import Img from './Img';

import type { AccountToken } from '../lib/daemon-rpc';
import type { SupportReportForm } from '../redux/support/actions';
type SupportState = {
  email: string,
  message: string,
  savedReport: ?string,
  sendState: 'INITIAL' | 'CONFIRM_NO_EMAIL' | 'LOADING' | 'SUCCESS' | 'FAILED',
};

export type SupportProps = {
  defaultEmail: string,
  defaultMessage: string,
  accountHistory: Array<AccountToken>,
  onClose: () => void,
  viewLog: (path: string) => void,
  saveReportForm: (form: SupportReportForm) => void,
  clearReportForm: () => void,
  collectProblemReport: (accountsToRedact: Array<string>) => Promise<string>,
  sendProblemReport: (email: string, message: string, savedReport: string) => Promise<void>,
};

export default class Support extends Component<SupportProps, SupportState> {
  state = {
    email: '',
    message: '',
    savedReport: null,
    sendState: 'INITIAL',
  };

  _collectLogPromise: ?Promise<string>;

  constructor(props: SupportProps) {
    super(props);

    // seed initial data from props
    this.state.email = props.defaultEmail;
    this.state.message = props.defaultMessage;
  }

  validate() {
    return this.state.message.trim().length > 0;
  }

  onChangeEmail = (email: string) => {
    this.setState({ email: email }, () => {
      this._saveFormData();
    });
  };

  onChangeDescription = (description: string) => {
    this.setState({ message: description }, () => {
      this._saveFormData();
    });
  };

  onViewLog = async (): Promise<void> => {
    try {
      const reportPath = await this._collectLog();
      this.props.viewLog(reportPath);
    } catch (error) {
      // TODO: handle error
    }
  };

  _saveFormData() {
    this.props.saveReportForm({
      email: this.state.email,
      message: this.state.message,
    });
  }

  async _collectLog(): Promise<string> {
    if (this._collectLogPromise) {
      return this._collectLogPromise;
    } else {
      const collectPromise = this.props.collectProblemReport(this.props.accountHistory);

      // save promise to prevent subsequent requests
      this._collectLogPromise = collectPromise;

      try {
        const reportPath = await collectPromise;
        return new Promise((resolve) => {
          this.setState({ savedReport: reportPath }, () => resolve(reportPath));
        });
      } catch (error) {
        this._collectLogPromise = null;

        throw error;
      }
    }
  }

  onSend = async (): Promise<void> => {
    if (this.state.sendState === 'INITIAL' && this.state.email.length === 0) {
      return new Promise((resolve) => {
        this.setState({ sendState: 'CONFIRM_NO_EMAIL' }, () => resolve());
      });
    } else {
      try {
        await this._sendReport();
      } catch (error) {
        // No-op
      }
    }
  };

  _sendReport(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.setState({ sendState: 'LOADING' }, async () => {
        try {
          const { email, message } = this.state;
          const reportPath = await this._collectLog();
          await this.props.sendProblemReport(email, message, reportPath);
          this.props.clearReportForm();
          this.setState({ sendState: 'SUCCESS' }, () => {
            resolve();
          });
        } catch (error) {
          this.setState({ sendState: 'FAILED' }, () => {
            reject(error);
          });
        }
      });
    });
  }

  render() {
    const { sendState } = this.state;
    const header = (
      <SettingsHeader>
        <HeaderTitle>Report a problem</HeaderTitle>
        {(sendState === 'INITIAL' || sendState === 'CONFIRM_NO_EMAIL') && (
          <HeaderSubTitle>
            {
              "To help you more effectively, your app's log file will be attached to this message. Your data will remain secure and private, as it is anonymised before being sent over an encrypted channel."
            }
          </HeaderSubTitle>
        )}
      </SettingsHeader>
    );

    const content = this._renderContent();

    return (
      <Layout>
        <Container>
          <View style={styles.support}>
            <NavigationBar>
              <BackBarItem action={this.props.onClose} title={'Settings'} />
            </NavigationBar>

            <View style={styles.support__container}>
              {header}

              {content}
            </View>
          </View>
        </Container>
      </Layout>
    );
  }

  _renderContent() {
    switch (this.state.sendState) {
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
    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row_email}>
            <TextInput
              style={styles.support__form_email}
              placeholder="Your email (optional)"
              defaultValue={this.state.email}
              onChangeText={this.onChangeEmail}
              keyboardType="email-address"
            />
          </View>
          <View style={styles.support__form_row_message}>
            <View style={styles.support__form_message_scroll_wrap}>
              <TextInput
                style={styles.support__form_message}
                placeholder="Describe your problem"
                defaultValue={this.state.message}
                multiline={true}
                onChangeText={this.onChangeDescription}
                testName="support__form_message"
              />
            </View>
          </View>
          <View style={styles.support__footer}>
            {this.state.sendState === 'CONFIRM_NO_EMAIL'
              ? this._renderNoEmailWarning()
              : this._renderActionButtons()}
          </View>
        </View>
      </View>
    );
  }

  _renderNoEmailWarning() {
    return (
      <View>
        <Text style={styles.support__no_email_warning}>
          You are about to send the problem report without a way for us to get back to you. If you
          want an answer to your report you will have to enter an email address.
        </Text>
        <AppButton.GreenButton
          disabled={!this.validate()}
          onPress={this.onSend}
          testName="support__send_logs">
          {'Send anyway'}
        </AppButton.GreenButton>
      </View>
    );
  }

  _renderActionButtons() {
    return (
      <View>
        <AppButton.BlueButton
          style={styles.view_logs_button}
          onPress={this.onViewLog}
          testName="support__view_logs">
          <AppButton.Label>View app logs</AppButton.Label>
          <Img source="icon-extLink" height={16} width={16} />
        </AppButton.BlueButton>
        <AppButton.GreenButton
          disabled={!this.validate()}
          onPress={this.onSend}
          testName="support__send_logs">
          Send
        </AppButton.GreenButton>
      </View>
    );
  }

  _renderLoading() {
    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row}>
            <View style={styles.support__status_icon}>
              <Img source="icon-spinner" height={60} width={60} alt="" />
            </View>
            <View style={styles.support__status_security__secure}>{'SECURE CONNECTION'}</View>
            <Text style={styles.support__send_status}>{'Sending...'}</Text>
          </View>
        </View>
      </View>
    );
  }

  _renderSent() {
    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row}>
            <View style={styles.support__status_icon}>
              <Img source="icon-success" height={60} width={60} alt="" />
            </View>
            <Text style={styles.support__status_security__secure}>{'SECURE CONNECTION'}</Text>
            <Text style={styles.support__send_status}>{'Sent'}</Text>

            <Text style={styles.support__sent_message}>Thanks! We will look into this.</Text>
            {this.state.email.trim().length > 0 ? (
              <Text style={styles.support__sent_message}>
                If needed we will contact you on {'\u00A0'}
                <Text style={styles.support__sent_email}>{this.state.email}</Text>
              </Text>
            ) : null}
          </View>
        </View>
      </View>
    );
  }

  _renderFailed() {
    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row}>
            <View style={styles.support__status_icon}>
              <Img source="icon-fail" height={60} width={60} alt="" />
            </View>
            <Text style={styles.support__status_security__secure}>{'SECURE CONNECTION'}</Text>
            <Text style={styles.support__send_status}>{'Failed to send'}</Text>
          </View>
        </View>
        <View style={styles.support__footer}>
          <AppButton.BlueButton
            style={styles.edit_message_button}
            onPress={() => this.setState({ sendState: 'INITIAL' })}>
            {'Edit message'}
          </AppButton.BlueButton>
          <AppButton.GreenButton onPress={this.onSend} testName="support__send_logs">
            Try again
          </AppButton.GreenButton>
        </View>
      </View>
    );
  }
}
