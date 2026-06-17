import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { type DialogProps } from '../../../../lib/components/dialog';
import { FailureDialog } from '../../../failure-dialog';
import { useDeviceListItemContext } from '../../DeviceListItemContext';

export type ErrorDialogProps = DialogProps;

export function ErrorDialog({ open, ...props }: ErrorDialogProps) {
  const { resetError } = useDeviceListItemContext();

  const handleOpenChange = React.useCallback(
    (open: boolean) => {
      if (!open) {
        resetError();
      }
    },
    [resetError],
  );

  return (
    <FailureDialog open={open} onOpenChange={handleOpenChange} {...props}>
      <FailureDialog.Text>
        {messages.pgettext('device-management', 'Failed to remove device')}
      </FailureDialog.Text>
      <FailureDialog.CloseButton>
        <FailureDialog.CloseButton.Text>{messages.gettext('Close')}</FailureDialog.CloseButton.Text>
      </FailureDialog.CloseButton>
    </FailureDialog>
  );
}
