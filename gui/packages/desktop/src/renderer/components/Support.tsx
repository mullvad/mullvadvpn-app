import * as React from 'react';
import { Component, Text, View, TextInput } from 'reactxp';
import {
  ImageView,
  SettingsHeader,
  HeaderTitle,
  HeaderSubTitle,
  ModalContainer,
  ModalContent,
  ModalAlert,
} from '@mullvad/components';
import * as AppButton from './AppButton';
import { Layout, Container } from './Layout';
import { NavigationBar, BackBarItem } from './NavigationBar';
import styles from './SupportStyles';

import { AccountToken } from '../../shared/daemon-rpc-types';
import { SupportReportForm } from '../redux/support/actions';

enum SendState {
  Initial,
  Confirm,
  Loading,
  Success,
  Failed,
}

type SupportState = {
  email: string;
  message: string;
  savedReport?: string;
  sendState: SendState;
};

type SupportProps = {
  defaultEmail: string;
  defaultMessage: string;
  accountHistory: Array<AccountToken>;
  isOffline: boolean;
  onClose: () => void;
  viewLog: (path: string) => void;
  saveReportForm: (form: SupportReportForm) => void;
  clearReportForm: () => void;
  collectProblemReport: (accountsToRedact: Array<string>) => Promise<string>;
  sendProblemReport: (email: string, message: string, savedReport: string) => Promise<void>;
};

export default class Support extends Component<SupportProps, SupportState> {
  state = {
    email: '',
    message: '',
    savedReport: undefined,
    sendState: SendState.Initial,
  };

  _collectLogPromise?: Promise<string>;

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
        this._collectLogPromise = undefined;

        throw error;
      }
    }
  }

  onSend = async (): Promise<void> => {
    switch (this.state.sendState) {
      case SendState.Initial:
        if (this.state.email.length === 0) {
          this.setState({ sendState: SendState.Confirm });
        } else {
          try {
            await this._sendReport();
          } catch (error) {
            // No-op
          }
        }
        return Promise.resolve();

      case SendState.Confirm:
        try {
          await this._sendReport();
        } catch (error) {
          // No-op
        }
        return Promise.resolve();

      default:
        break;
    }

    return Promise.resolve();
  };

  onCancelConfirmation = () => {
    this.setState({ sendState: SendState.Initial });
  };

  _sendReport(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.setState({ sendState: SendState.Loading }, async () => {
        try {
          const { email, message } = this.state;
          const reportPath = await this._collectLog();
          await this.props.sendProblemReport(email, message, reportPath);
          this.props.clearReportForm();
          this.setState({ sendState: SendState.Success }, () => {
            resolve();
          });
        } catch (error) {
          this.setState({ sendState: SendState.Failed }, () => {
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
        {(sendState === SendState.Initial || sendState === SendState.Confirm) && (
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
          <ModalContainer>
            <ModalContent>
              <View style={styles.support}>
                <NavigationBar>
                  <BackBarItem action={this.props.onClose}>Settings</BackBarItem>
                </NavigationBar>
                <View style={styles.support__container}>
                  {header}
                  {content}
                </View>
              </View>
            </ModalContent>
            {sendState === SendState.Confirm ? (
              <ModalAlert>{this._renderConfirm()}</ModalAlert>
            ) : (
              undefined
            )}
          </ModalContainer>
        </Container>
      </Layout>
    );
  }

  _renderContent() {
    switch (this.state.sendState) {
      case SendState.Initial:
      case SendState.Confirm:
        return this._renderForm();
      case SendState.Loading:
        return this._renderLoading();
      case SendState.Success:
        return this._renderSent();
      case SendState.Failed:
        return this._renderFailed();
      default:
        return null;
    }
  }

  _renderConfirm() {
    return <ConfirmNoEmailDialog onConfirm={this.onSend} onDismiss={this.onCancelConfirmation} />;
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
              />
            </View>
          </View>
          <View style={styles.support__footer}>
            <AppButton.BlueButton style={styles.view_logs_button} onPress={this.onViewLog}>
              <AppButton.Label>View app logs</AppButton.Label>
              <AppButton.Icon source="icon-extLink" height={16} width={16} />
            </AppButton.BlueButton>
            <AppButton.GreenButton disabled={!this.validate()} onPress={this.onSend}>
              Send
            </AppButton.GreenButton>
          </View>
        </View>
      </View>
    );
  }

  _renderLoading() {
    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row}>
            <View style={styles.support__status_icon}>
              <ImageView source="icon-spinner" height={60} width={60} />
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
              <ImageView source="icon-success" height={60} width={60} />
            </View>
            <Text style={styles.support__status_security__secure}>{'SECURE CONNECTION'}</Text>
            <Text style={styles.support__send_status}>{'Sent'}</Text>

            <Text style={styles.support__sent_message}>Thanks! We will look into this.</Text>
            {this.state.email.trim().length > 0 ? (
              <Text style={styles.support__sent_message}>
                {'If needed we will contact you on '}
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
              <ImageView source="icon-fail" height={60} width={60} />
            </View>
            <Text style={styles.support__status_security__secure}>{'SECURE CONNECTION'}</Text>
            <Text style={styles.support__send_status}>{'Failed to send'}</Text>
            <Text style={styles.support__sent_message}>
              {
                "You may need to go back to the app's main screen and click Disconnect before trying again. Don't worry, the information you entered will remain in the form."
              }
            </Text>
          </View>
        </View>
        <View style={styles.support__footer}>
          <AppButton.BlueButton
            style={styles.edit_message_button}
            onPress={() => this.setState({ sendState: SendState.Initial })}>
            {'Edit message'}
          </AppButton.BlueButton>
          <AppButton.GreenButton onPress={this.onSend}>Try again</AppButton.GreenButton>
        </View>
      </View>
    );
  }
}

type ConfirmNoEmailDialogProps = {
  onConfirm: () => void;
  onDismiss: () => void;
};

class ConfirmNoEmailDialog extends Component<ConfirmNoEmailDialogProps> {
  render() {
    return (
      <View style={styles.confirm_no_email_background}>
        <View style={styles.confirm_no_email_dialog}>
          <Text style={styles.confirm_no_email_warning}>
            You are about to send the problem report without a way for us to get back to you. If you
            want an answer to your report you will have to enter an email address.
          </Text>
          <AppButton.GreenButton onPress={this.props.onConfirm}>
            {'Send anyway'}
          </AppButton.GreenButton>
          <AppButton.RedButton onPress={this._dismiss} style={styles.confirm_no_email_back_button}>
            {'Back'}
          </AppButton.RedButton>
        </View>
      </View>
    );
  }

  _confirm = () => {
    this.props.onConfirm();
  };

  _dismiss = () => {
    this.props.onDismiss();
  };
}
