import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { TransitionType, useHistory } from '../../../../../lib/history';
import { StatusDialog } from '../../../../status-dialog';
import { useTroubleshootingSteps } from './hooks';

export type TroubleshootingModalProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export function TroubleshootingModal({ open, onOpenChange }: TroubleshootingModalProps) {
  const { push } = useHistory();

  const openSendProblemReport = React.useCallback(() => {
    onOpenChange(false);
    push(RoutePath.problemReport, { transition: TransitionType.show });
  }, [onOpenChange, push]);

  const steps = useTroubleshootingSteps();

  return (
    <StatusDialog variant="info" open={open} onOpenChange={onOpenChange}>
      <FlexColumn>
        <StatusDialog.Text>
          {
            // TRANSLATORS: Message in troubleshooting modal when the background process failed to start.
            messages.pgettext(
              'launch-view',
              'The Mullvad background process failed to start. The background process is responsible for the security, kill switch, and the VPN tunnel. Please try:',
            )
          }
        </StatusDialog.Text>
        <StatusDialog.Text>
          <StatusDialog.List>{steps}</StatusDialog.List>
        </StatusDialog.Text>
      </FlexColumn>
      <StatusDialog.Text>
        {
          // TRANSLATORS: Message in troubleshooting modal advising user to send a problem report if the steps do not work.
          messages.pgettext(
            'launch-view',
            'If these steps do not work please send a problem report.',
          )
        }
      </StatusDialog.Text>
      <StatusDialog.ButtonGroup>
        <StatusDialog.Button onClick={openSendProblemReport} variant="success">
          <StatusDialog.Button.Text>
            {
              // TRANSLATORS: Button label for sending a problem report.
              messages.pgettext('launch-view', 'Send problem report')
            }
          </StatusDialog.Button.Text>
        </StatusDialog.Button>
        <StatusDialog.CloseButton>
          <StatusDialog.CloseButton.Text>{messages.gettext('Back')}</StatusDialog.CloseButton.Text>
        </StatusDialog.CloseButton>
      </StatusDialog.ButtonGroup>
    </StatusDialog>
  );
}
