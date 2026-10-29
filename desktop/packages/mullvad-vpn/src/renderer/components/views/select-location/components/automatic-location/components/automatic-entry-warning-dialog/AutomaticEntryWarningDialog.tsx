import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { StatusDialog, StatusDialogProps } from '../../../../../../status-dialog';
import { useHandleSelectEntryLocation } from '../../../location-lists/hooks';

type AutomaticEntryWarningDialogProps = Omit<StatusDialogProps, 'children' | 'variant'>;

export function AutomaticEntryWarningDialog({
  open,
  onOpenChange,
  ...props
}: AutomaticEntryWarningDialogProps) {
  const handleSelectEntryLocation = useHandleSelectEntryLocation();

  const handleClickContinue = React.useCallback(() => {
    void handleSelectEntryLocation('any');
  }, [handleSelectEntryLocation]);

  return (
    <StatusDialog variant="info" open={open} onOpenChange={onOpenChange} {...props}>
      <StatusDialog.Text>
        {
          // TRANSLATORS: Text in dialog explaining that selecting the “Automatic” location option
          // TRANSLATORS: will override the entry filters the user has selected.
          messages.pgettext(
            'select-location-view',
            'You currently have entry filters applied. Selecting the “Automatic” location, the app will ignore filter settings for the entry server.',
          )
        }
      </StatusDialog.Text>
      <StatusDialog.ButtonGroup>
        <StatusDialog.Button onClick={handleClickContinue}>
          <StatusDialog.Button.Text>{messages.gettext('Continue')}</StatusDialog.Button.Text>
        </StatusDialog.Button>
        <StatusDialog.CloseButton>
          <StatusDialog.CloseButton.Text>
            {messages.gettext('Cancel')}
          </StatusDialog.CloseButton.Text>
        </StatusDialog.CloseButton>
      </StatusDialog.ButtonGroup>
    </StatusDialog>
  );
}
