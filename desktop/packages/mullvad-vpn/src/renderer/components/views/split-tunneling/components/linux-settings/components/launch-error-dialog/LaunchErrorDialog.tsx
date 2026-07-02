import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { WarningDialog } from '../../../../../../warning-dialog';
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
    <WarningDialog open={hasBrowseError} onOpenChange={handleOnOpenChange}>
      <WarningDialog.Text>
        {sprintf(
          // TRANSLATORS: Error message showed in a dialog when an application fails to launch.
          messages.pgettext(
            'split-tunneling-view',
            'Unable to launch selection. %(detailedErrorMessage)s',
          ),
          { detailedErrorMessage: browseError },
        )}
      </WarningDialog.Text>
      <WarningDialog.CloseButton>
        <WarningDialog.CloseButton.Text>{messages.gettext('Close')}</WarningDialog.CloseButton.Text>
      </WarningDialog.CloseButton>
    </WarningDialog>
  );
}
