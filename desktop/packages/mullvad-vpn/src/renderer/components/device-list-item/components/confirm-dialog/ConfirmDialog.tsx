import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { FlexColumn } from '../../../../lib/components/flex-column';
import { formatHtml } from '../../../../lib/html-formatter';
import { formatDeviceName } from '../../../../lib/utils';
import { StatusDialog } from '../../../status-dialog';
import { useDeviceListItemContext } from '../../DeviceListItemContext';
import { useHandleRemoveDevice } from './hooks';

export function ConfirmDialog({ open }: { open: boolean }) {
  const { device, hideConfirmDialog, deleting } = useDeviceListItemContext();
  const handleRemoveDevice = useHandleRemoveDevice();

  return (
    <StatusDialog variant="info" open={open} onOpenChange={hideConfirmDialog}>
      <FlexColumn>
        <StatusDialog.Text>
          {formatHtml(
            sprintf(
              // TRANSLATORS: Text displayed above button which logs out another device.
              // TRANSLATORS: The text enclosed in "<b></b>" will appear bold.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(deviceName)s - The name of the device to log out.
              messages.pgettext('device-management', 'Remove <em>%(deviceName)s?</em>'),
              { deviceName: formatDeviceName(device.name) },
            ),
          )}
        </StatusDialog.Text>
        <StatusDialog.Text variant="labelTinySemiBold" color="whiteAlpha60">
          {messages.pgettext(
            'device-management',
            'The device will be removed from the list and logged out.',
          )}
        </StatusDialog.Text>
      </FlexColumn>
      <StatusDialog.ButtonGroup>
        <StatusDialog.Button onClick={handleRemoveDevice} disabled={deleting}>
          <StatusDialog.Button.Text>
            {
              // TRANSLATORS: Button label for confirming removing a device.
              messages.pgettext('device-management', 'Remove')
            }
          </StatusDialog.Button.Text>
        </StatusDialog.Button>
        <StatusDialog.Button onClick={hideConfirmDialog} disabled={deleting}>
          <StatusDialog.Button.Text>{messages.gettext('Back')}</StatusDialog.Button.Text>
        </StatusDialog.Button>
      </StatusDialog.ButtonGroup>
    </StatusDialog>
  );
}
