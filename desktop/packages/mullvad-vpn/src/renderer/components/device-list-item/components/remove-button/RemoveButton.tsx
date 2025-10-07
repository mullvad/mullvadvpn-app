import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton } from '../../../../lib/components';
import { useDeviceListItemContext } from '../..';

export function RemoveButton() {
  const { device, deleting, showConfirmDialog } = useDeviceListItemContext();

  return (
    <IconButton
      variant="secondary"
      onClick={showConfirmDialog}
      disabled={deleting}
      aria-label={sprintf(
        // TRANSLATORS: Button action description provided to accessibility tools such as screen
        // TRANSLATORS: readers.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(deviceName)s - The device name to remove.
        messages.pgettext('accessibility', 'Remove device named %(deviceName)s'),
        { deviceName: device.name },
      )}>
      <IconButton.Icon icon="cross-circle" />
    </IconButton>
  );
}
