import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { RoutePath } from '../../shared/routes';
import { useAppContext } from '../context';
import { Button } from '../lib/components';
import { colors } from '../lib/foundations';
import { TransitionType, useHistory } from '../lib/history';
import { useBoolean } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import { measurements, tinyText } from './common-styles';
import ErrorView from './ErrorView';
import { Footer } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage, ModalMessageList } from './Modal';

export default function Launch() {
  const daemonAllowed = useSelector((state) => state.userInterface.daemonAllowed);
  const footer = daemonAllowed === false ? <MacOsPermissionFooter /> : <DefaultFooter />;

  return (
    <ErrorView footer={footer}>
      {messages.pgettext('launch-view', 'Connecting to Mullvad system service...')}
    </ErrorView>
  );
}

const StyledFooter = styled(Footer)({
  backgroundColor: colors.blue,
  transition: 'opacity 250ms ease-in-out',
});

const StyledFooterInner = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  backgroundColor: colors.darkBlue,
  borderRadius: '8px',
  margin: 0,
  padding: '16px',
});

const StyledFooterMessage = styled.span(tinyText, {
  color: colors.white,
  margin: `8px 0 ${measurements.buttonVerticalMargin} 0`,
});

function MacOsPermissionFooter() {
  const { showLaunchDaemonSettings } = useAppContext();

  const openSettings = useCallback(async () => {
    await showLaunchDaemonSettings();
  }, [showLaunchDaemonSettings]);

  return (
    <StyledFooter>
      <StyledFooterInner>
        <StyledFooterMessage>
          {messages.pgettext(
            'launch-view',
            'Permission for the Mullvad VPN service has been revoked. Please go to System Settings and allow Mullvad VPN under the “Allow in the Background” setting.',
          )}
        </StyledFooterMessage>
        <Button onClick={openSettings}>
          <Button.Text>
            {
              // TRANSLATORS: Button label for system settings.
              messages.gettext('Go to System Settings')
            }
          </Button.Text>
        </Button>
      </StyledFooterInner>
    </StyledFooter>
  );
}

function DefaultFooter() {
  const { push } = useHistory();
  const [dialogVisible, showDialog, hideDialog] = useBoolean();

  const openSendProblemReport = useCallback(() => {
    hideDialog();
    push(RoutePath.problemReport, { transition: TransitionType.show });
  }, [hideDialog, push]);

  return (
    <>
      <StyledFooter>
        <StyledFooterInner>
          <StyledFooterMessage>
            {messages.pgettext(
              'launch-view',
              'Unable to contact the Mullvad system service, your connection might be unsecure. Please troubleshoot or send a problem report by clicking the Learn more button.',
            )}
          </StyledFooterMessage>
          <Button onClick={showDialog}>
            <Button.Text>{messages.gettext('Learn more')}</Button.Text>
          </Button>
        </StyledFooterInner>
      </StyledFooter>
      <ModalAlert
        isOpen={dialogVisible}
        type={ModalAlertType.info}
        close={hideDialog}
        buttons={[
          <Button variant="success" key="problem-report" onClick={openSendProblemReport}>
            <Button.Text>
              {
                // TRANSLATORS: Button label for problem report view.
                messages.pgettext('launch-view', 'Send problem report')
              }
            </Button.Text>
          </Button>,
          <Button key="back" onClick={hideDialog}>
            <Button.Text>{messages.gettext('Back')}</Button.Text>
          </Button>,
        ]}>
        <ModalMessage>
          {messages.pgettext(
            'launch-view',
            'The system service component of the app hasn’t started or can’t be contacted. The system service is responsible for the security, kill switch, and the VPN tunnel. To troubleshoot please try:',
          )}
        </ModalMessage>
        <ModalMessage>
          <ModalMessageList>
            <li>{messages.pgettext('launch-view', 'Restarting your computer.')}</li>
            <li>{messages.pgettext('launch-view', 'Reinstalling the app.')}</li>
            <li>{messages.pgettext('launch-view', 'Disable third party antivirus software.')}</li>
          </ModalMessageList>
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'launch-view',
            'If these steps do not work please send a problem report.',
          )}
        </ModalMessage>
      </ModalAlert>
    </>
  );
}
