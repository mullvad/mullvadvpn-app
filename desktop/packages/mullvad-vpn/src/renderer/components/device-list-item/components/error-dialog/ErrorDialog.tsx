import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { type DialogProps } from '../../../../lib/components/dialog';
import { StatusDialog } from '../../../status-dialog';
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
    <StatusDialog variant="failure" open={open} onOpenChange={handleOpenChange} {...props}>
      <StatusDialog.Text>
        {messages.pgettext('device-management', 'Failed to remove device')}
      </StatusDialog.Text>
      <StatusDialog.CloseButton>
        <StatusDialog.CloseButton.Text>{messages.gettext('Close')}</StatusDialog.CloseButton.Text>
      </StatusDialog.CloseButton>
    </StatusDialog>
  );
}
