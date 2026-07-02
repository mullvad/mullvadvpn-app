import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { FlexColumn } from '../../../../lib/components/flex-column';
import { formatHtml } from '../../../../lib/html-formatter';
import { formatDeviceName } from '../../../../lib/utils';
import { InfoDialog } from '../../../info-dialog';
import { useDeviceListItemContext } from '../../DeviceListItemContext';
import { useHandleRemoveDevice } from './hooks';

export function ConfirmDialog({ open }: { open: boolean }) {
  const { device, hideConfirmDialog, deleting } = useDeviceListItemContext();
  const handleRemoveDevice = useHandleRemoveDevice();

  return (
    <InfoDialog open={open} onOpenChange={hideConfirmDialog}>
      <FlexColumn>
        <InfoDialog.Text>
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
        </InfoDialog.Text>
        <InfoDialog.Text variant="labelTinySemiBold" color="whiteAlpha60">
          {messages.pgettext(
            'device-management',
            'The device will be removed from the list and logged out.',
          )}
        </InfoDialog.Text>
      </FlexColumn>
      <InfoDialog.ButtonGroup>
        <InfoDialog.Button onClick={handleRemoveDevice} disabled={deleting}>
          <InfoDialog.Button.Text>
            {
              // TRANSLATORS: Button label for confirming removing a device.
              messages.pgettext('device-management', 'Remove')
            }
          </InfoDialog.Button.Text>
        </InfoDialog.Button>
        <InfoDialog.Button onClick={hideConfirmDialog} disabled={deleting}>
          <InfoDialog.Button.Text>{messages.gettext('Back')}</InfoDialog.Button.Text>
        </InfoDialog.Button>
      </InfoDialog.ButtonGroup>
    </InfoDialog>
  );
}
