import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../shared/constants';
import { messages } from '../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../lib/components';
import { Colors } from '../../../../../../../lib/foundations';
import { ModalAlert, ModalAlertType } from '../../../../../../Modal';
import { useLinuxSettingsContext } from '../../LinuxSettingsContext';
import { useHideUnsupportedDialog } from './hooks';

export function UnsupportedDialog() {
  const { showUnsupportedDialog } = useLinuxSettingsContext();
  const hideUnsupportedDialog = useHideUnsupportedDialog();
  const iconColor: Colors = 'white';

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

  const buttons = [
    <Button key="cancel" onClick={hideUnsupportedDialog}>
      <Button.Text>{messages.gettext('Got it!')}</Button.Text>
    </Button>,
  ];

  return (
    <ModalAlert
      isOpen={showUnsupportedDialog}
      type={ModalAlertType.info}
      iconColor={iconColor}
      message={unsupportedMessage}
      buttons={buttons}
      close={hideUnsupportedDialog}
    />
  );
}
