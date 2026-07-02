import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { TransitionType, useHistory } from '../../../../../lib/history';
import { InfoDialog } from '../../../../info-dialog';
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
    <InfoDialog open={open} onOpenChange={onOpenChange}>
      <FlexColumn>
        <InfoDialog.Text>
          {
            // TRANSLATORS: Message in troubleshooting modal when the background process failed to start.
            messages.pgettext(
              'launch-view',
              'The Mullvad background process failed to start. The background process is responsible for the security, kill switch, and the VPN tunnel. Please try:',
            )
          }
        </InfoDialog.Text>
        <InfoDialog.Text>
          <InfoDialog.List>{steps}</InfoDialog.List>
        </InfoDialog.Text>
      </FlexColumn>
      <InfoDialog.Text>
        {
          // TRANSLATORS: Message in troubleshooting modal advising user to send a problem report if the steps do not work.
          messages.pgettext(
            'launch-view',
            'If these steps do not work please send a problem report.',
          )
        }
      </InfoDialog.Text>
      <InfoDialog.ButtonGroup>
        <InfoDialog.Button onClick={openSendProblemReport} variant="success">
          <InfoDialog.Button.Text>
            {
              // TRANSLATORS: Button label for sending a problem report.
              messages.pgettext('launch-view', 'Send problem report')
            }
          </InfoDialog.Button.Text>
        </InfoDialog.Button>
        <InfoDialog.CloseButton>
          <InfoDialog.CloseButton.Text>{messages.gettext('Back')}</InfoDialog.CloseButton.Text>
        </InfoDialog.CloseButton>
      </InfoDialog.ButtonGroup>
    </InfoDialog>
  );
}
