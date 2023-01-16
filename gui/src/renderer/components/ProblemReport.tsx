import {
  ChangeEvent,
  createContext,
  Dispatch,
  ReactNode,
  SetStateAction,
  useCallback,
  useContext,
  useEffect,
  useMemo,
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
  return (
    <ProblemReportContextProvider>
      <ProblemReportComponent />
    </ProblemReportContextProvider>
  );
}

function ProblemReportComponent() {
  const history = useHistory();

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
            <Header />
            <Content />
          </StyledContentContainer>

          <NoEmailDialog />
          <OutdatedVersionWarningDialog />
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function Header() {
  const { sendState } = useProblemReportContext();

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

function Content() {
  const { sendState } = useProblemReportContext();

  switch (sendState) {
    case SendState.initial:
    case SendState.confirm:
      return <Form />;
    case SendState.sending:
      return <Sending />;
    case SendState.success:
      return <Sent />;
    case SendState.failed:
      return <Failed />;
    default:
      return null;
  }
}

function Form() {
  const { viewLog } = useAppContext();
  const { email, setEmail, message, setMessage, onSend } = useProblemReportContext();
  const { collectLog } = useCollectLog();

  const [disableActions, setDisableActions] = useState(false);

  const onViewLog = useCallback(async () => {
    setDisableActions(true);

    try {
      const reportId = await collectLog();
      await viewLog(reportId);
    } catch (error) {
      // TODO: handle error
    } finally {
      setDisableActions(false);
    }
  }, []);

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

function Sent() {
  const { email } = useProblemReportContext();

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

function Failed() {
  const { setSendState, onSend } = useProblemReportContext();

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

function NoEmailDialog() {
  const { sendState, setSendState, onSend } = useProblemReportContext();

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

const useCollectLog = () => {
  const { collectProblemReport } = useAppContext();
  const accountHistory = useSelector((state) => state.account.accountHistory);

  const collectLogPromise = useRef<Promise<string>>();

  const collectLog = useCallback(async (): Promise<string> => {
    if (collectLogPromise.current) {
      return collectLogPromise.current;
    } else {
      const collectPromise = collectProblemReport(accountHistory);
      // save promise to prevent subsequent requests
      collectLogPromise.current = collectPromise;

      try {
        const reportId = await collectPromise;
        return reportId;
      } catch (error) {
        collectLogPromise.current = undefined;
        throw error;
      }
    }
  }, [collectLogPromise]);

  return { collectLog };
};

type ProblemReportContextType = {
  sendState: SendState;
  setSendState: Dispatch<SetStateAction<SendState>>;
  email: string;
  setEmail: Dispatch<SetStateAction<string>>;
  message: string;
  setMessage: Dispatch<SetStateAction<string>>;
  onSend: () => Promise<void>;
};

const ProblemReportContext = createContext<ProblemReportContextType | undefined>(undefined);

const ProblemReportContextProvider = ({ children }: { children: ReactNode }) => {
  const { sendProblemReport } = useAppContext();
  const { clearReportForm, saveReportForm } = useActions(support);

  const { email: defaultEmail, message: defaultMessage } = useSelector((state) => state.support);

  const { collectLog } = useCollectLog();

  const [sendState, setSendState] = useState(SendState.initial);
  const [email, setEmail] = useState(defaultEmail);
  const [message, setMessage] = useState(defaultMessage);

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

  const onSend = useCallback(async () => {
    if (sendState === SendState.initial && email.length === 0) {
      setSendState(SendState.confirm);
    } else if (
      sendState === SendState.initial ||
      sendState === SendState.confirm ||
      sendState === SendState.failed
    ) {
      try {
        setSendState(SendState.sending);
        await sendReport();
      } catch (error) {
        // No-op
      }
    }
  }, [email, sendReport, sendState]);

  /**
   * Save the form whenever email or message gets updated
   */
  useEffect(() => {
    saveReportForm({ email, message });
  }, [email, message]);

  const value: ProblemReportContextType = useMemo(
    () => ({ sendState, setSendState, email, setEmail, message, setMessage, onSend }),
    [sendState, setSendState, email, setEmail, message, setMessage, onSend],
  );
  return <ProblemReportContext.Provider value={value}>{children}</ProblemReportContext.Provider>;
};

const useProblemReportContext = () => {
  const context = useContext(ProblemReportContext);
  if (!context) {
    throw new Error('useProblemReportContext must be used within a ProblemReportContextProvider');
  }
  return context;
};
