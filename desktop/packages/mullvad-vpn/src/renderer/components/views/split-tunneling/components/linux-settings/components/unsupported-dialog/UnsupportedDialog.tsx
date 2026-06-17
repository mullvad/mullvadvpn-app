import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../shared/constants';
import { messages } from '../../../../../../../../shared/gettext';
import { InfoDialog } from '../../../../../../info-dialog';
import { useLinuxSettingsContext } from '../../LinuxSettingsContext';

export function UnsupportedDialog() {
  const { showUnsupportedDialog, setShowUnsupportedDialog } = useLinuxSettingsContext();

  const unsupportedMessage = sprintf(
    // TRANSLATORS: Information about split tunneling being unavailable due to
    // TRANSLATORS: missing support in the user's operating system.
    // TRANSLATORS: Available placeholders:
    // TRANSLATORS: %(splitTunneling)s - will be replaced with Split tunneling
    messages.pgettext(
      'split-tunneling-view',
      'To use %(splitTunneling)s, please change to a Linux kernel version that supports cgroup v1.',
    ),
    {
      splitTunneling: strings.splitTunneling,
    },
  );

  return (
    <InfoDialog open={showUnsupportedDialog} onOpenChange={setShowUnsupportedDialog}>
      <InfoDialog.Text>{unsupportedMessage}</InfoDialog.Text>
      <InfoDialog.CloseButton>
        <InfoDialog.CloseButton.Text>{messages.gettext('Got it!')}</InfoDialog.CloseButton.Text>
      </InfoDialog.CloseButton>
    </InfoDialog>
  );
}
