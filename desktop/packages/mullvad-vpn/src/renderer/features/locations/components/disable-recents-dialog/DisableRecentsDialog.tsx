import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { StatusDialog, type StatusDialogProps } from '../../../../components/status-dialog';
import { useRecents } from '../../hooks';

export type DisableRecentsDialogProps = Omit<StatusDialogProps, 'variant'>;

export function DisableRecentsDialog({ onOpenChange, ...props }: DisableRecentsDialogProps) {
  const { setEnabledRecents } = useRecents();

  const disableRecents = React.useCallback(async () => {
    await setEnabledRecents(false);
    onOpenChange?.(false);
  }, [onOpenChange, setEnabledRecents]);

  const handleCancel = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

  return (
    <StatusDialog variant="info" onOpenChange={onOpenChange} {...props}>
      <StatusDialog.Text>
        {messages.pgettext('locations-feature', 'Disabling recents will also clear history.')}
      </StatusDialog.Text>
      <StatusDialog.ButtonGroup>
        <StatusDialog.Button color="destructive" onClick={disableRecents}>
          <StatusDialog.Button.Text>{messages.gettext('Disable')}</StatusDialog.Button.Text>
        </StatusDialog.Button>
        <StatusDialog.Button key="cancel" onClick={handleCancel}>
          <StatusDialog.Button.Text>{messages.gettext('Cancel')}</StatusDialog.Button.Text>
        </StatusDialog.Button>
      </StatusDialog.ButtonGroup>
    </StatusDialog>
  );
}
