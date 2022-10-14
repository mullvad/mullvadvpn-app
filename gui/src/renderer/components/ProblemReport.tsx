import {
  ChangeEvent,
  Dispatch,
  ReactNode,
  SetStateAction,
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';

import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import { useHistory } from '../lib/history';
import { useSelector } from '../redux/store';
import support from '../redux/support/actions';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { Footer, Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationBar, NavigationItems, TitleBarItem } from './NavigationBar';
import {
  StyledContent,
  StyledContentContainer,
  StyledEmail,
  StyledEmailInput,
  StyledForm,
  StyledFormEmailRow,
  StyledFormMessageRow,
  StyledMessageInput,
  StyledSendStatus,
  StyledSentMessage,
  StyledStatusIcon,
  StyledThanks,
} from './ProblemReportStyles';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';

enum SendState {
  initial,
  confirm,
  sending,
  success,
  failed,
}

export default function ProblemReport() {
  const history = useHistory();
  const { collectProblemReport, sendProblemReport, viewLog } = useAppContext();
  const { clearReportForm, saveReportForm } = useActions(support);

  const { email: defaultEmail, message: defaultMessage } = useSelector((state) => state.support);
  const accountHistory = useSelector((state) => state.account.accountHistory);

  const [sendState, setSendState] = useState(SendState.initial);
  const [email, setEmail] = useState(defaultEmail);
  const [message, setMessage] = useState(defaultMessage);
  const [disableActions, setDisableActions] = useState(false);

  const sendReport = useCallback(async () => {
    try {
      const reportId = await collectLog();
      await sendProblemReport(email, message, reportId);
      clearReportForm();
      setSendState(SendState.success);
    } catch (error) {
      setSendState(SendState.failed);
    }
  }, [email, message]);

  /**
   * Save the form whenever email or message gets updated
   */
  useEffect(() => {
    saveReportForm({ email, message });
  }, [email, message]);

  /**
   * Listen for changes to sendState,
   * when it is set to sending, send the report
   */
  useEffect(() => {
    if (sendState === SendState.sending) {
      void sendReport();
    }
  }, [sendState]);

  /**
   * A bit awkward, but when actions are disabled,
   * we use that as a trigger to collect and view the log
   */
  useEffect(() => {
    const collectAndViewLog = async () => {
      const reportId = await collectLog();
      await viewLog(reportId);
    };

    if (disableActions) {
      try {
        void collectAndViewLog();
      } catch (error) {
        // TODO: handle error
      } finally {
        setDisableActions(false);
      }
    }
  }, [disableActions]);

  const collectLogPromise = useRef<Promise<string>>();

  const collectLog = async (): Promise<string> => {
    if (collectLogPromise.current) {
      return collectLogPromise.current;
    } else {
      const collectPromise = collectProblemReport(accountHistory);
      // save promise to prevent subsequent requests
      collectLogPromise.current = collectPromise;

      try {
        const reportId = await collectPromise;
        return new Promise((resolve) => {
          resolve(reportId);
        });
      } catch (error) {
        collectLogPromise.current = undefined;
        throw error;
      }
    }
  };

  const onViewLog = useCallback(() => {
    setDisableActions(true);
  }, []);

  const onSend = useCallback(() => {
    if (sendState === SendState.initial && email.length === 0) {
      setSendState(SendState.confirm);
    } else if (
      sendState === SendState.initial ||
      sendState === SendState.confirm ||
      sendState === SendState.failed
    ) {
      try {
        setSendState(SendState.sending);
      } catch (error) {
        // No-op
      }
    }
  }, [email]);

  const renderContent = () => {
    switch (sendState) {
      case SendState.initial:
      case SendState.confirm:
        return (
          <Form
            email={email}
            setEmail={setEmail}
            message={message}
            setMessage={setMessage}
            onSend={onSend}
            disableActions={disableActions}
            onViewLog={onViewLog}
          />
        );
      case SendState.sending:
        return <Sending />;
      case SendState.success:
        return <Sent email={email} />;
      case SendState.failed:
        return <Failed setSendState={setSendState} onSend={onSend} />;
      default:
        return null;
    }
  };

  const content = renderContent();

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationBar>
            <NavigationItems>
              <TitleBarItem>
                {
                  // TRANSLATORS: Title label in navigation bar
                  messages.pgettext('support-view', 'Report a problem')
                }
              </TitleBarItem>
            </NavigationItems>
          </NavigationBar>
          <StyledContentContainer>
            <Header sendState={sendState} />
            {content}
          </StyledContentContainer>

          <NoEmailDialog sendState={sendState} setSendState={setSendState} onSend={onSend} />
          <OutdatedVersionWarningDialog />
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

type HeaderProps = {
  sendState: SendState;
};

function Header({ sendState }: HeaderProps) {
  return (
    <SettingsHeader>
      <HeaderTitle>{messages.pgettext('support-view', 'Report a problem')}</HeaderTitle>
      {(sendState === SendState.initial || sendState === SendState.confirm) && (
        <HeaderSubTitle>
          {messages.pgettext(
            'support-view',
            'To help you more effectively, your app’s log file will be attached to this message. Your data will remain secure and private, as it is anonymised before being sent over an encrypted channel.',
          )}
        </HeaderSubTitle>
      )}
    </SettingsHeader>
  );
}

type FormProps = {
  email: string;
  setEmail: (email: string) => void;
  disableActions: boolean;
  message: string;
  setMessage: (message: string) => void;
  onSend: () => void;
  onViewLog: () => void;
};

function Form({
  email,
  disableActions,
  message,
  onSend,
  setEmail,
  setMessage,
  onViewLog,
}: FormProps) {
  const onChangeEmail = useCallback((event: ChangeEvent<HTMLInputElement>) => {
    setEmail(event.target.value);
  }, []);

  const onChangeDescription = useCallback((event: ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(event.target.value);
  }, []);

  const validate = () => message.trim().length > 0;

  return (
    <StyledContent>
      <StyledForm>
        <StyledFormEmailRow>
          <StyledEmailInput
            placeholder={messages.pgettext('support-view', 'Your email (optional)')}
            defaultValue={email}
            onChange={onChangeEmail}
          />
        </StyledFormEmailRow>
        <StyledFormMessageRow>
          <StyledMessageInput
            placeholder={messages.pgettext(
              'support-view',
              'Please describe your problem in English or Swedish.',
            )}
            defaultValue={message}
            onChange={onChangeDescription}
          />
        </StyledFormMessageRow>
      </StyledForm>
      <Footer>
        <AriaDescriptionGroup>
          <AriaDescribed>
            <AppButton.ButtonGroup>
              <AppButton.BlueButton onClick={onViewLog} disabled={disableActions}>
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
              </AppButton.BlueButton>
            </AppButton.ButtonGroup>
          </AriaDescribed>
        </AriaDescriptionGroup>
        <AppButton.GreenButton disabled={!validate() || disableActions} onClick={onSend}>
          {messages.pgettext('support-view', 'Send')}
        </AppButton.GreenButton>
      </Footer>
    </StyledContent>
  );
}

function Sending() {
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

type SentProps = {
  email: string;
};

function Sent({ email }: SentProps) {
  const reachBackMessage: ReactNode[] =
    // TRANSLATORS: The message displayed to the user after submitting the problem report, given that the user left his or her email for us to reach back.
    // TRANSLATORS: Available placeholders:
    // TRANSLATORS: %(email)s
    messages
      .pgettext('support-view', 'If needed we will contact you at %(email)s')
      .split('%(email)s', 2);
  reachBackMessage.splice(1, 0, <StyledEmail key="email">{email}</StyledEmail>);

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
        {email.trim().length > 0 ? <StyledSentMessage>{reachBackMessage}</StyledSentMessage> : null}
      </StyledForm>
    </StyledContent>
  );
}

type FailedProps = {
  setSendState: Dispatch<SetStateAction<SendState>>;
  onSend: () => void;
};

function Failed({ setSendState, onSend }: FailedProps) {
  const handleEditMessage = useCallback(() => {
    setSendState(SendState.initial);
  }, []);

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
            'If you exit the form and try again later, the information you already entered will still be here.',
          )}
        </StyledSentMessage>
      </StyledForm>
      <Footer>
        <AppButton.ButtonGroup>
          <AppButton.BlueButton onClick={handleEditMessage}>
            {messages.pgettext('support-view', 'Edit message')}
          </AppButton.BlueButton>
          <AppButton.GreenButton onClick={onSend}>
            {messages.pgettext('support-view', 'Try again')}
          </AppButton.GreenButton>
        </AppButton.ButtonGroup>
      </Footer>
    </StyledContent>
  );
}

type NoEmailDialogProps = {
  sendState: SendState;
  onSend: () => void;
  setSendState: Dispatch<SetStateAction<SendState>>;
};

function NoEmailDialog({ sendState, onSend, setSendState }: NoEmailDialogProps) {
  const message = messages.pgettext(
    'support-view',
    'You are about to send the problem report without a way for us to get back to you. If you want an answer to your report you will have to enter an email address.',
  );

  const onCancelNoEmailDialog = useCallback(() => {
    setSendState(SendState.initial);
  }, []);

  return (
    <ModalAlert
      isOpen={sendState === SendState.confirm}
      type={ModalAlertType.warning}
      message={message}
      buttons={[
        <AppButton.RedButton key="proceed" onClick={onSend}>
          {messages.pgettext('support-view', 'Send anyway')}
        </AppButton.RedButton>,
        <AppButton.BlueButton key="cancel" onClick={onCancelNoEmailDialog}>
          {messages.gettext('Back')}
        </AppButton.BlueButton>,
      ]}
      close={onCancelNoEmailDialog}
    />
  );
}

function OutdatedVersionWarningDialog() {
  const history = useHistory();
  const { openUrl } = useAppContext();

  const isOffline = useSelector((state) => state.connection.isBlocked);
  const suggestedIsBeta = useSelector((state) => state.version.suggestedIsBeta ?? false);
  const outdatedVersion = useSelector((state) => !!state.version.suggestedUpgrade);

  const [showOutdatedVersionWarning, setShowOutdatedVersionWarning] = useState(outdatedVersion);

  const acknowledgeOutdatedVersion = useCallback(() => {
    setShowOutdatedVersionWarning(false);
  }, []);

  const openDownloadLink = useCallback(async () => {
    await openUrl(suggestedIsBeta ? links.betaDownload : links.download);
  }, [suggestedIsBeta]);

  const onClose = useCallback(() => history.pop(), [history.pop]);

  const outdatedVersionCancel = useCallback(() => {
    acknowledgeOutdatedVersion();
    onClose();
  }, [onClose]);

  const message = messages.pgettext(
    'support-view',
    'You are using an old version of the app. Please upgrade and see if the problem still exists before sending a report.',
  );

  return (
    <ModalAlert
      isOpen={showOutdatedVersionWarning}
      type={ModalAlertType.warning}
      message={message}
      buttons={[
        <AriaDescriptionGroup key="upgrade">
          <AriaDescribed>
            <AppButton.GreenButton disabled={isOffline} onClick={openDownloadLink}>
              <AppButton.Label>{messages.pgettext('support-view', 'Upgrade app')}</AppButton.Label>
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
        <AppButton.RedButton key="proceed" onClick={acknowledgeOutdatedVersion}>
          {messages.pgettext('support-view', 'Continue anyway')}
        </AppButton.RedButton>,
        <AppButton.BlueButton key="cancel" onClick={outdatedVersionCancel}>
          {messages.gettext('Cancel')}
        </AppButton.BlueButton>,
      ]}
      close={onClose}
    />
  );
}
