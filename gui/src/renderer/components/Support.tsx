import * as React from 'react';
import { Component, Text, TextInput, View } from 'reactxp';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import * as AppButton from './AppButton';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import { ModalAlert, ModalAlertType, ModalContainer } from './Modal';
import { BackBarItem, NavigationBar, NavigationItems } from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import styles from './SupportStyles';

import { AccountToken } from '../../shared/daemon-rpc-types';
import { ISupportReportForm } from '../redux/support/actions';

enum SendState {
  Initial,
  Confirm,
  Sending,
  Success,
  Failed,
}

interface ISupportState {
  email: string;
  message: string;
  savedReport?: string;
  sendState: SendState;
  disableActions: boolean;
  showOutdatedVersionWarning: boolean;
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
  outdatedVersion: boolean;
  onExternalLink: (url: string) => void;
}

export default class Support extends Component<ISupportProps, ISupportState> {
  public state = {
    email: '',
    message: '',
    savedReport: undefined,
    sendState: SendState.Initial,
    disableActions: false,
    showOutdatedVersionWarning: false,
  };

  private collectLogPromise?: Promise<string>;

  constructor(props: ISupportProps) {
    super(props);

    // seed initial data from props
    this.state.email = props.defaultEmail;
    this.state.message = props.defaultMessage;
    this.state.showOutdatedVersionWarning = props.outdatedVersion;
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

  public onViewLog = () => {
    this.performWithActionsDisabled(async () => {
      try {
        const reportPath = await this.collectLog();
        this.props.viewLog(reportPath);
      } catch (error) {
        // TODO: handle error
      }
    });
  };

  public onSend = async (): Promise<void> => {
    const sendState = this.state.sendState;
    if (sendState === SendState.Initial && this.state.email.length === 0) {
      this.setState({ sendState: SendState.Confirm });
    } else if (sendState === SendState.Initial || sendState === SendState.Confirm) {
      try {
        await this.sendReport();
      } catch (error) {
        // No-op
      }
    }
  };

  public onCancelNoEmailDialog = () => {
    this.setState({ sendState: SendState.Initial });
  };

  public render() {
    const { sendState, showOutdatedVersionWarning } = this.state;
    const header = (
      <SettingsHeader>
        <HeaderTitle>{messages.pgettext('support-view', 'Report a problem')}</HeaderTitle>
        {(sendState === SendState.Initial || sendState === SendState.Confirm) && (
          <HeaderSubTitle>
            {messages.pgettext(
              'support-view',
              "To help you more effectively, your app's log file will be attached to this message. Your data will remain secure and private, as it is anonymised before being sent over an encrypted channel.",
            )}
          </HeaderSubTitle>
        )}
      </SettingsHeader>
    );

    const content = this.renderContent();

    return (
      <ModalContainer>
        <Layout>
          <Container>
            <View style={styles.support}>
              <NavigationBar>
                <NavigationItems>
                  <BackBarItem action={this.props.onClose}>
                    {
                      // TRANSLATORS: Back button in navigation bar
                      messages.pgettext('navigation-bar', 'Settings')
                    }
                  </BackBarItem>
                </NavigationItems>
              </NavigationBar>
              <View style={styles.support__container}>
                {header}
                {content}
              </View>
            </View>
            {sendState === SendState.Confirm && this.renderNoEmailDialog()}
            {showOutdatedVersionWarning && this.renderOutdateVersionWarningDialog()}
          </Container>
        </Layout>
      </ModalContainer>
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
      this.setState({ sendState: SendState.Sending }, async () => {
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
      case SendState.Sending:
        return this.renderSending();
      case SendState.Success:
        return this.renderSent();
      case SendState.Failed:
        return this.renderFailed();
      default:
        return null;
    }
  }

  private renderNoEmailDialog() {
    const message = messages.pgettext(
      'support-view',
      'You are about to send the problem report without a way for us to get back to you. If you want an answer to your report you will have to enter an email address.',
    );
    return (
      <ModalAlert
        type={ModalAlertType.Warning}
        message={message}
        buttons={[
          <AppButton.RedButton key="proceed" onPress={this.onSend}>
            {messages.pgettext('support-view', 'Send anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="cancel" onPress={this.onCancelNoEmailDialog}>
            {messages.pgettext('support-view', 'Back')}
          </AppButton.BlueButton>,
        ]}
      />
    );
  }

  private acknowledgeOutdateVersion = () => {
    this.setState({ showOutdatedVersionWarning: false });
  };

  private openDownloadLink = () => this.props.onExternalLink(links.download);

  private renderOutdateVersionWarningDialog() {
    const message = messages.pgettext(
      'support-view',
      'You are using an old version of the app. Please upgrade and see if the problem still exists before sending a report.',
    );
    return (
      <ModalAlert
        type={ModalAlertType.Warning}
        message={message}
        buttons={[
          <AppButton.GreenButton
            key="upgrade"
            disabled={this.props.isOffline}
            onPress={this.openDownloadLink}>
            <AppButton.Label>{messages.pgettext('support-view', 'Upgrade app')}</AppButton.Label>
            <AppButton.Icon height={16} width={16} source="icon-extLink" />
          </AppButton.GreenButton>,
          <AppButton.RedButton key="proceed" onPress={this.acknowledgeOutdateVersion}>
            {messages.pgettext('support-view', 'Continue anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="cancel" onPress={this.props.onClose}>
            {messages.pgettext('support-view', 'Cancel')}
          </AppButton.BlueButton>,
        ]}
      />
    );
  }

  private renderForm() {
    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row_email}>
            <TextInput
              style={styles.support__form_email}
              placeholder={messages.pgettext('support-view', 'Your email (optional)')}
              defaultValue={this.state.email}
              onChangeText={this.onChangeEmail}
              keyboardType="email-address"
            />
          </View>
          <View style={styles.support__form_row_message}>
            <View style={styles.support__form_message_scroll_wrap}>
              <TextInput
                style={styles.support__form_message}
                placeholder={messages.pgettext('support-view', 'Describe your problem')}
                defaultValue={this.state.message}
                multiline={true}
                onChangeText={this.onChangeDescription}
              />
            </View>
          </View>
          <View style={styles.support__footer}>
            <AppButton.BlueButton
              style={styles.view_logs_button}
              onPress={this.onViewLog}
              disabled={this.state.disableActions}>
              <AppButton.Label>
                {messages.pgettext('support-view', 'View app logs')}
              </AppButton.Label>
              <AppButton.Icon source="icon-extLink" height={16} width={16} />
            </AppButton.BlueButton>
            <AppButton.GreenButton
              disabled={!this.validate() || this.state.disableActions}
              onPress={this.onSend}>
              {messages.pgettext('support-view', 'Send')}
            </AppButton.GreenButton>
          </View>
        </View>
      </View>
    );
  }

  private renderSending() {
    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row}>
            <View style={styles.support__status_icon}>
              <ImageView source="icon-spinner" height={60} width={60} />
            </View>
            <View style={styles.support__status_security__secure}>
              {messages.gettext('SECURE CONNECTION')}
            </View>
            <Text style={styles.support__send_status}>
              {messages.pgettext('support-view', 'Sending...')}
            </Text>
          </View>
        </View>
      </View>
    );
  }

  private renderSent() {
    const reachBackMessage: React.ReactNodeArray =
      // TRANSLATORS: The message displayed to the user after submitting the problem report, given that the user left his or her email for us to reach back.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(email)s
      messages
        .pgettext('support-view', 'If needed we will contact you on %(email)s')
        .split('%(email)s', 2);
    reachBackMessage.splice(
      1,
      0,
      <Text key={'email'} style={styles.support__sent_email}>
        {this.state.email}
      </Text>,
    );

    return (
      <View style={styles.support__content}>
        <View style={styles.support__form}>
          <View style={styles.support__form_row}>
            <View style={styles.support__status_icon}>
              <ImageView source="icon-success" height={60} width={60} />
            </View>
            <Text style={styles.support__status_security__secure}>
              {messages.gettext('SECURE CONNECTION')}
            </Text>
            <Text style={styles.support__send_status}>
              {messages.pgettext('support-view', 'Sent')}
            </Text>

            <Text style={styles.support__sent_message}>
              {messages.pgettext('support-view', 'Thanks! We will look into this.')}
            </Text>
            {this.state.email.trim().length > 0 ? (
              <Text style={styles.support__sent_message}>{reachBackMessage}</Text>
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
            <Text style={styles.support__status_security__secure}>
              {messages.gettext('SECURE CONNECTION')}
            </Text>
            <Text style={styles.support__send_status}>
              {messages.pgettext('support-view', 'Failed to send')}
            </Text>
            <Text style={styles.support__sent_message}>
              {messages.pgettext(
                'support-view',
                "You may need to go back to the app's main screen and click Disconnect before trying again. Don't worry, the information you entered will remain in the form.",
              )}
            </Text>
          </View>
        </View>
        <View style={styles.support__footer}>
          <AppButton.BlueButton style={styles.edit_message_button} onPress={this.handleEditMessage}>
            {messages.pgettext('support-view', 'Edit message')}
          </AppButton.BlueButton>
          <AppButton.GreenButton onPress={this.onSend}>
            {messages.pgettext('support-view', 'Try again')}
          </AppButton.GreenButton>
        </View>
      </View>
    );
  }

  private handleEditMessage = () => {
    this.setState({ sendState: SendState.Initial });
  };

  private performWithActionsDisabled(work: () => Promise<void>) {
    this.setState({ disableActions: true }, async () => {
      try {
        await work();
      } catch {
        // TODO: handle error
      }
      this.setState({ disableActions: false });
    });
  }
}
