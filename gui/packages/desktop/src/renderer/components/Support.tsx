import {
  HeaderSubTitle,
  HeaderTitle,
  ImageView,
  ModalAlert,
  ModalContainer,
  ModalContent,
  SettingsHeader,
} from '@mullvad/components';
import * as React from 'react';
import { Component, Text, TextInput, View } from 'reactxp';
import * as AppButton from './AppButton';
import { Container, Layout } from './Layout';
import { BackBarItem, NavigationBar } from './NavigationBar';
import styles from './SupportStyles';

import { AccountToken } from '../../shared/daemon-rpc-types';
import { ISupportReportForm } from '../redux/support/actions';

enum SendState {
  Initial,
  Confirm,
  Loading,
  Success,
  Failed,
}

interface ISupportState {
  email: string;
  message: string;
  savedReport?: string;
  sendState: SendState;
}

interface ISupportProps {
  defaultEmail: string;
  defaultMessage: string;
  accountHistory: AccountToken[];
  isOffline: boolean;
  onClose: () => void;
  viewLog: (path: string) => void;
  saveReportForm: (form: ISupportReportForm) => void;
  clearReportForm: () => void;
  collectProblemReport: (accountsToRedact: string[]) => Promise<string>;
  sendProblemReport: (email: string, message: string, savedReport: string) => Promise<void>;
}

export default class Support extends Component<ISupportProps, ISupportState> {
  public state = {
    email: '',
    message: '',
    savedReport: undefined,
    sendState: SendState.Initial,
  };

  private collectLogPromise?: Promise<string>;

  constructor(props: ISupportProps) {
    super(props);

    // seed initial data from props
    this.state.email = props.defaultEmail;
    this.state.message = props.defaultMessage;
  }

  public validate() {
    return this.state.message.trim().length > 0;
  }

  public onChangeEmail = (email: string) => {
    this.setState({ email }, () => {
      this.saveFormData();
    });
  };

  public onChangeDescription = (description: string) => {
    this.setState({ message: description }, () => {
      this.saveFormData();
    });
  };

  public onViewLog = async (): Promise<void> => {
    try {
      const reportPath = await this.collectLog();
      this.props.viewLog(reportPath);
    } catch (error) {
      // TODO: handle error
    }
  };

  public onSend = async (): Promise<void> => {
    switch (this.state.sendState) {
      case SendState.Initial:
        if (this.state.email.length === 0) {
          this.setState({ sendState: SendState.Confirm });
        } else {
          try {
            await this.sendReport();
          } catch (error) {
            // No-op
          }
        }
        return Promise.resolve();

      case SendState.Confirm:
        try {
          await this.sendReport();
        } catch (error) {
          // No-op
        }
        return Promise.resolve();

      default:
        break;
    }

    return Promise.resolve();
  };

  public onCancelConfirmation = () => {
    this.setState({ sendState: SendState.Initial });
  };

  public render() {
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

    const content = this.renderContent();

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
              <ModalAlert>{this.renderConfirm()}</ModalAlert>
            ) : (
              undefined
            )}
          </ModalContainer>
        </Container>
      </Layout>
    );
  }

  private saveFormData() {
    this.props.saveReportForm({
      email: this.state.email,
      message: this.state.message,
    });
  }

  private async collectLog(): Promise<string> {
    if (this.collectLogPromise) {
      return this.collectLogPromise;
    } else {
      const collectPromise = this.props.collectProblemReport(this.props.accountHistory);

      // save promise to prevent subsequent requests
      this.collectLogPromise = collectPromise;

      try {
        const reportPath = await collectPromise;
        return new Promise((resolve) => {
          this.setState({ savedReport: reportPath }, () => resolve(reportPath));
        });
      } catch (error) {
        this.collectLogPromise = undefined;

        throw error;
      }
    }
  }

  private sendReport(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.setState({ sendState: SendState.Loading }, async () => {
        try {
          const { email, message } = this.state;
          const reportPath = await this.collectLog();
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

  private renderContent() {
    switch (this.state.sendState) {
      case SendState.Initial:
      case SendState.Confirm:
        return this.renderForm();
      case SendState.Loading:
        return this.renderLoading();
      case SendState.Success:
        return this.renderSent();
      case SendState.Failed:
        return this.renderFailed();
      default:
        return null;
    }
  }

  private renderConfirm() {
    return <ConfirmNoEmailDialog onConfirm={this.onSend} onDismiss={this.onCancelConfirmation} />;
  }

  private renderForm() {
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

  private renderLoading() {
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

  private renderSent() {
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

  private renderFailed() {
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
          <AppButton.BlueButton style={styles.edit_message_button} onPress={this.handleEditMessage}>
            {'Edit message'}
          </AppButton.BlueButton>
          <AppButton.GreenButton onPress={this.onSend}>Try again</AppButton.GreenButton>
        </View>
      </View>
    );
  }

  private handleEditMessage = () => {
    this.setState({ sendState: SendState.Initial });
  };
}

interface IConfirmNoEmailDialogProps {
  onConfirm: () => void;
  onDismiss: () => void;
}

class ConfirmNoEmailDialog extends Component<IConfirmNoEmailDialogProps> {
  public render() {
    return (
      <View style={styles.confirm_no_email_background}>
        <View style={styles.confirm_no_email_dialog}>
          <Text style={styles.confirm_no_email_warning}>
            You are about to send the problem report without a way for us to get back to you. If you
            want an answer to your report you will have to enter an email address.
          </Text>
          <AppButton.GreenButton onPress={this.confirm}>{'Send anyway'}</AppButton.GreenButton>
          <AppButton.RedButton onPress={this.dismiss} style={styles.confirm_no_email_back_button}>
            {'Back'}
          </AppButton.RedButton>
        </View>
      </View>
    );
  }

  private confirm = () => {
    this.props.onConfirm();
  };

  private dismiss = () => {
    this.props.onDismiss();
  };
}
