import * as React from 'react';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import ImageView from './ImageView';
import { Layout } from './Layout';
import { ModalAlert, ModalAlertType, ModalContainer } from './Modal';
import { BackBarItem, NavigationBar, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import {
  StyledBlueButton,
  StyledContainer,
  StyledContent,
  StyledContentContainer,
  StyledEmail,
  StyledEmailInput,
  StyledFooter,
  StyledForm,
  StyledFormEmailRow,
  StyledFormMessageRow,
  StyledSendStatus,
  StyledSentMessage,
  StyledStatusIcon,
  StyledThanks,
  StyledMessageInput,
} from './SupportStyles';

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

export default class Support extends React.Component<ISupportProps, ISupportState> {
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

  public onChangeEmail = (event: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ email: event.target.value }, () => {
      this.saveFormData();
    });
  };

  public onChangeDescription = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    this.setState({ message: event.target.value }, () => {
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
    } else if (
      sendState === SendState.Initial ||
      sendState === SendState.Confirm ||
      sendState === SendState.Failed
    ) {
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
          <StyledContainer>
            <NavigationBar>
              <NavigationItems>
                <BackBarItem action={this.props.onClose}>
                  {
                    // TRANSLATORS: Back button in navigation bar
                    messages.pgettext('navigation-bar', 'Settings')
                  }
                </BackBarItem>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('support-view', 'Report a problem')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>
            <StyledContentContainer>
              {header}
              {content}
            </StyledContentContainer>

            {sendState === SendState.Confirm && this.renderNoEmailDialog()}
            {showOutdatedVersionWarning && this.renderOutdateVersionWarningDialog()}
          </StyledContainer>
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
          <AppButton.RedButton key="proceed" onClick={this.onSend}>
            {messages.pgettext('support-view', 'Send anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="cancel" onClick={this.onCancelNoEmailDialog}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={this.onCancelNoEmailDialog}
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
          <AriaDescriptionGroup key="upgrade">
            <AriaDescribed>
              <AppButton.GreenButton
                disabled={this.props.isOffline}
                onClick={this.openDownloadLink}>
                <AppButton.Label>
                  {messages.pgettext('support-view', 'Upgrade app')}
                </AppButton.Label>
                <AriaDescription>
                  <AppButton.Icon
                    height={16}
                    width={16}
                    source="icon-extLink"
                    aria-label={messages.pgettext('accessibility', 'Opens externally')}
                  />
                </AriaDescription>
              </AppButton.GreenButton>
            </AriaDescribed>
          </AriaDescriptionGroup>,
          <AppButton.RedButton key="proceed" onClick={this.acknowledgeOutdateVersion}>
            {messages.pgettext('support-view', 'Continue anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="cancel" onClick={this.props.onClose}>
            {messages.gettext('Cancel')}
          </AppButton.BlueButton>,
        ]}
        close={this.props.onClose}
      />
    );
  }

  private renderForm() {
    return (
      <StyledContent>
        <StyledForm>
          <StyledFormEmailRow>
            <StyledEmailInput
              placeholder={messages.pgettext('support-view', 'Your email (optional)')}
              defaultValue={this.state.email}
              onChange={this.onChangeEmail}
            />
          </StyledFormEmailRow>
          <StyledFormMessageRow>
            <StyledMessageInput
              placeholder={messages.pgettext('support-view', 'Describe your problem')}
              defaultValue={this.state.message}
              onChange={this.onChangeDescription}
            />
          </StyledFormMessageRow>
        </StyledForm>
        <StyledFooter>
          <AriaDescriptionGroup>
            <AriaDescribed>
              <StyledBlueButton onClick={this.onViewLog} disabled={this.state.disableActions}>
                <AppButton.Label>
                  {messages.pgettext('support-view', 'View app logs')}
                </AppButton.Label>
                <AriaDescription>
                  <AppButton.Icon
                    source="icon-extLink"
                    height={16}
                    width={16}
                    aria-label={messages.pgettext('accessibility', 'Opens externally')}
                  />
                </AriaDescription>
              </StyledBlueButton>
            </AriaDescribed>
          </AriaDescriptionGroup>
          <AppButton.GreenButton
            disabled={!this.validate() || this.state.disableActions}
            onClick={this.onSend}>
            {messages.pgettext('support-view', 'Send')}
          </AppButton.GreenButton>
        </StyledFooter>
      </StyledContent>
    );
  }

  private renderSending() {
    return (
      <StyledContent>
        <StyledForm>
          <StyledStatusIcon>
            <ImageView source="icon-spinner" height={60} width={60} />
          </StyledStatusIcon>
          <StyledSendStatus>{messages.pgettext('support-view', 'Sending...')}</StyledSendStatus>
        </StyledForm>
      </StyledContent>
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
    reachBackMessage.splice(1, 0, <StyledEmail key="email">{this.state.email}</StyledEmail>);

    return (
      <StyledContent>
        <StyledForm>
          <StyledStatusIcon>
            <ImageView source="icon-success" height={60} width={60} />
          </StyledStatusIcon>
          <StyledSendStatus>{messages.pgettext('support-view', 'Sent')}</StyledSendStatus>

          <StyledSentMessage>
            <StyledThanks>{messages.pgettext('support-view', 'Thanks!')} </StyledThanks>
            {messages.pgettext('support-view', 'We will look into this.')}
          </StyledSentMessage>
          {this.state.email.trim().length > 0 ? (
            <StyledSentMessage>{reachBackMessage}</StyledSentMessage>
          ) : null}
        </StyledForm>
      </StyledContent>
    );
  }

  private renderFailed() {
    return (
      <StyledContent>
        <StyledForm>
          <StyledStatusIcon>
            <ImageView source="icon-fail" height={60} width={60} />
          </StyledStatusIcon>
          <StyledSendStatus>{messages.pgettext('support-view', 'Failed to send')}</StyledSendStatus>
          <StyledSentMessage>
            {messages.pgettext(
              'support-view',
              "You may need to go back to the app's main screen and click Disconnect before trying again. Don't worry, the information you entered will remain in the form.",
            )}
          </StyledSentMessage>
        </StyledForm>
        <StyledFooter>
          <StyledBlueButton onClick={this.handleEditMessage}>
            {messages.pgettext('support-view', 'Edit message')}
          </StyledBlueButton>
          <AppButton.GreenButton onClick={this.onSend}>
            {messages.pgettext('support-view', 'Try again')}
          </AppButton.GreenButton>
        </StyledFooter>
      </StyledContent>
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
