import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { Dialog } from '../../../../lib/components/dialog';
import { FlexColumn } from '../../../../lib/components/flex-column';
import { formatHtml } from '../../../../lib/html-formatter';
import { formatDeviceName } from '../../../../lib/utils';
import { useDeviceListItemContext } from '../../DeviceListItemContext';
import { useHandleRemoveDevice } from './hooks';

export function ConfirmDialog({ isOpen }: { isOpen: boolean }) {
  const { device, hideConfirmDialog, deleting } = useDeviceListItemContext();
  const handleRemoveDevice = useHandleRemoveDevice();

  return (
    <Dialog open={isOpen} onOpenChange={hideConfirmDialog}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="info-circle" />
            <FlexColumn>
              <Dialog.Text>
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
              </Dialog.Text>
              <Dialog.Text variant="labelTinySemiBold" color="whiteAlpha60">
                {messages.pgettext(
                  'device-management',
                  'The device will be removed from the list and logged out.',
                )}
              </Dialog.Text>
            </FlexColumn>
            <Dialog.ButtonGroup>
              <Dialog.Button onClick={handleRemoveDevice} disabled={deleting}>
                <Dialog.Button.Text>
                  {
                    // TRANSLATORS: Button label for confirming removing a device.
                    messages.pgettext('device-management', 'Remove')
                  }
                </Dialog.Button.Text>
              </Dialog.Button>
              <Dialog.Button onClick={hideConfirmDialog} disabled={deleting}>
                <Dialog.Button.Text>{messages.gettext('Back')}</Dialog.Button.Text>
              </Dialog.Button>
            </Dialog.ButtonGroup>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}
