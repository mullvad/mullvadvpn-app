import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { StatusDialog, type StatusDialogProps } from '../../../../components/status-dialog';

type EntryFiltersWarningDialogProps = Omit<StatusDialogProps, 'onOpenChange' | 'variant'> & {
  onCancel: () => void;
  onContinue: () => void;
};

export function EntryFiltersWarningDialog({
  onContinue,
  onCancel,
  ...props
}: EntryFiltersWarningDialogProps) {
  const handleOpenChange = React.useCallback(() => {
    onCancel();
  }, [onCancel]);

  return (
    <StatusDialog variant="caution" onOpenChange={handleOpenChange} {...props}>
      <StatusDialog.Text>
        {messages.pgettext(
          'locations-feature',
          'You currently have entry filters applied. Switching to “When needed”, the app will ignore filter settings for the entry server that is being automatically selected.',
        )}
      </StatusDialog.Text>
      <StatusDialog.ButtonGroup>
        <StatusDialog.Button onClick={onContinue}>
          <StatusDialog.Button.Text>{messages.gettext('Continue')}</StatusDialog.Button.Text>
        </StatusDialog.Button>
        <StatusDialog.Button onClick={onCancel}>
          <StatusDialog.Button.Text>{messages.gettext('Cancel')}</StatusDialog.Button.Text>
        </StatusDialog.Button>
      </StatusDialog.ButtonGroup>
    </StatusDialog>
  );
}
