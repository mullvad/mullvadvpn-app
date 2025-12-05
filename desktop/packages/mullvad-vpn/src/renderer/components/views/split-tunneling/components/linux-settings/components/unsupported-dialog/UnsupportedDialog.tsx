import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../shared/constants';
import { messages } from '../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../lib/components';
import { ModalAlert, ModalAlertType } from '../../../../../../Modal';
import { useLinuxSettingsContext } from '../../LinuxSettingsContext';

export function UnsupportedDialog() {
  const { showUnsupportedDialog, setShowUnsupportedDialog } = useLinuxSettingsContext();
  const hideUnsupportedDialog = useCallback(() => {
    setShowUnsupportedDialog(false);
  }, [setShowUnsupportedDialog]);

  const unsupportedMessage = sprintf(
    // TRANSLATORS: Information about split tunneling being unavailable due to
    // TRANSLATORS: missing support in the user's operating system.
    // TRANSLATORS: Available placeholders:
    // TRANSLATORS: %(splitTunneling)s - will be replaced with Split tunneling
    messages.pgettext(
      'split-tunneling-view',
      'To use %(splitTunneling)s, please update to a Linux kernel version that supports cgroup v2.',
    ),
    {
      splitTunneling: strings.splitTunneling,
    },
  );

  const buttons = [
    <Button key="cancel" onClick={hideUnsupportedDialog}>
      <Button.Text>{messages.gettext('Got it!')}</Button.Text>
    </Button>,
  ];

  return (
    <ModalAlert
      isOpen={showUnsupportedDialog}
      type={ModalAlertType.info}
      message={unsupportedMessage}
      buttons={buttons}
      close={hideUnsupportedDialog}
    />
  );
}
