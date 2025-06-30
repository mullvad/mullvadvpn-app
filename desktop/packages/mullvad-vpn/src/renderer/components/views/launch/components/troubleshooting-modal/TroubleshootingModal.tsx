import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Button } from '../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../lib/history';
import { ModalAlert, ModalAlertType, ModalMessage, ModalMessageList } from '../../../../Modal';
import { useTroubleshootingSteps } from './hooks';

export type TroubleshootingModalProps = {
  isOpen: boolean;
  onClose: () => void;
};

const StyledModalMessage = styled(ModalMessage)`
  margin-top: 0;
`;

export function TroubleshootingModal({ isOpen, onClose }: TroubleshootingModalProps) {
  const { push } = useHistory();
  const openSendProblemReport = useCallback(() => {
    onClose();
    push(RoutePath.problemReport, { transition: TransitionType.show });
  }, [onClose, push]);

  const steps = useTroubleshootingSteps();

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
          <Button.Text>{messages.gettext('Back')}</Button.Text>
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
      <StyledModalMessage>
        <ModalMessageList>{steps}</ModalMessageList>
      </StyledModalMessage>
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
