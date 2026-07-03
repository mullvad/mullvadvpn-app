import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../shared/constants';
import { messages } from '../../../../../../../../shared/gettext';
import { StatusDialog } from '../../../../../../status-dialog';
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
    <StatusDialog
      variant="info"
      open={showUnsupportedDialog}
      onOpenChange={setShowUnsupportedDialog}>
      <StatusDialog.Text>{unsupportedMessage}</StatusDialog.Text>
      <StatusDialog.CloseButton>
        <StatusDialog.CloseButton.Text>{messages.gettext('Got it!')}</StatusDialog.CloseButton.Text>
      </StatusDialog.CloseButton>
    </StatusDialog>
  );
}
