import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Button } from '../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../lib/history';
import { ModalAlert, ModalAlertType, ModalMessage, ModalMessageList } from '../../../../Modal';

export type TroubleshootingModalProps = {
  isOpen: boolean;
  onClose: () => void;
};

export function TroubleshootingModal({ isOpen, onClose }: TroubleshootingModalProps) {
  const { push } = useHistory();
  const openSendProblemReport = useCallback(() => {
    onClose();
    push(RoutePath.problemReport, { transition: TransitionType.show });
  }, [onClose, push]);
  return (
    <ModalAlert
      isOpen={isOpen}
      type={ModalAlertType.info}
      close={onClose}
      buttons={[
        <Button variant="success" key="problem-report" onClick={openSendProblemReport}>
          <Button.Text>
            {
              // TRANSLATORS: Button label for sending a problem report.
              messages.pgettext('launch-view', 'Send problem report')
            }
          </Button.Text>
        </Button>,
        <Button key="back" onClick={onClose}>
          <Button.Text>
            {
              // TRANSLATORS: Button label for navigating back.
              messages.gettext('Back')
            }
          </Button.Text>
        </Button>,
      ]}>
      <ModalMessage>
        {
          // TRANSLATORS: Message in troubleshooting modal when the background process failed to start.
          messages.pgettext(
            'launch-view',
            'The Mullvad background process failed to start. The background process is responsible for the security, kill switch, and the VPN tunnel. Please try:',
          )
        }
      </ModalMessage>
      <ModalMessage>
        <ModalMessageList>
          <li>
            {
              // TRANSLATORS: List item in troubleshooting modal advising user to restart background process.
              messages.pgettext('launch-view', 'Restarting the Mullvad background process')
            }
          </li>
          <li>
            {
              // TRANSLATORS: List item in troubleshooting modal advising user to restart their computer.
              messages.pgettext('launch-view', 'Restarting your computer')
            }
          </li>
          <li>
            {
              // TRANSLATORS: List item in troubleshooting modal advising user to reinstall the app.
              messages.pgettext('launch-view', 'Reinstalling the app')
            }
          </li>
          <li>
            {
              // TRANSLATORS: List item in troubleshooting modal advising user disable third party antivirus.
              messages.pgettext('launch-view', 'Disabling third party antivirus software')
            }
          </li>
        </ModalMessageList>
      </ModalMessage>
      <ModalMessage>
        {
          // TRANSLATORS: Message in troubleshooting modal advising user to send a problem report if the steps do not work.
          messages.pgettext(
            'launch-view',
            'If these steps do not work please send a problem report.',
          )
        }
      </ModalMessage>
    </ModalAlert>
  );
}
