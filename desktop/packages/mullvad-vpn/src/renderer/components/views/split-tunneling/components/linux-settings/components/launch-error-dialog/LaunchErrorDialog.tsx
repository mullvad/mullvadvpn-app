import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../lib/components';
import { colors } from '../../../../../../../lib/foundations';
import { ModalAlert, ModalAlertType } from '../../../../../../Modal';
import { useLinuxSettingsContext } from '../../LinuxSettingsContext';
import { useHasBrowseError, useHideBrowseFailureDialog } from './hooks';

export function LaunchErrorDialog() {
  const { browseError } = useLinuxSettingsContext();
  const hasBrowseError = useHasBrowseError();
  const hideBrowseFailureDialog = useHideBrowseFailureDialog();

  return (
    <ModalAlert
      isOpen={hasBrowseError}
      type={ModalAlertType.warning}
      iconColor={colors.red}
      message={sprintf(
        // TRANSLATORS: Error message showed in a dialog when an application fails to launch.
        messages.pgettext(
          'split-tunneling-view',
          'Unable to launch selection. %(detailedErrorMessage)s',
        ),
        { detailedErrorMessage: browseError },
      )}
      buttons={[
        <Button key="close" onClick={hideBrowseFailureDialog}>
          <Button.Text>{messages.gettext('Close')}</Button.Text>
        </Button>,
      ]}
      close={hideBrowseFailureDialog}
    />
  );
}
