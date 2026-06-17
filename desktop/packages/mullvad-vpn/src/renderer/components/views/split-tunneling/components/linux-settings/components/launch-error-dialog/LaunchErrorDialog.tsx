import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { StatusDialog } from '../../../../../../status-dialog';
import { useLinuxSettingsContext } from '../../LinuxSettingsContext';
import { useHasBrowseError } from './hooks';

export function LaunchErrorDialog() {
  const { browseError, setBrowseError } = useLinuxSettingsContext();
  const hasBrowseError = useHasBrowseError();

  const handleOnOpenChange = React.useCallback(
    (open: boolean) => {
      if (!open) {
        setBrowseError(undefined);
      }
    },
    [setBrowseError],
  );

  return (
    <StatusDialog variant="warning" open={hasBrowseError} onOpenChange={handleOnOpenChange}>
      <StatusDialog.Text>
        {sprintf(
          // TRANSLATORS: Error message showed in a dialog when an application fails to launch.
          messages.pgettext(
            'split-tunneling-view',
            'Unable to launch selection. %(detailedErrorMessage)s',
          ),
          { detailedErrorMessage: browseError },
        )}
      </StatusDialog.Text>
      <StatusDialog.CloseButton>
        <StatusDialog.CloseButton.Text>{messages.gettext('Close')}</StatusDialog.CloseButton.Text>
      </StatusDialog.CloseButton>
    </StatusDialog>
  );
}
