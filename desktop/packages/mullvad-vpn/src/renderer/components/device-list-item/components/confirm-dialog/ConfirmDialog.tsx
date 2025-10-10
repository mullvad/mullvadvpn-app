import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { Button, Text } from '../../../../lib/components';
import { formatHtml } from '../../../../lib/html-formatter';
import { formatDeviceName } from '../../../../lib/utils';
import { IModalAlertProps, ModalAlert, ModalAlertType, ModalMessage } from '../../../Modal';
import { useDeviceListItemContext } from '../../DeviceListItemContext';
import { useHandleRemoveDevice } from './hooks';

export type ConfirmDialogProps = IModalAlertProps;

export function ConfirmDialog({ isOpen }: ConfirmDialogProps) {
  const { device, hideConfirmDialog, deleting } = useDeviceListItemContext();
  const handleRemoveDevice = useHandleRemoveDevice();

  return (
    <ModalAlert
      isOpen={isOpen}
      type={ModalAlertType.caution}
      buttons={[
        <Button key="remove" onClick={handleRemoveDevice} disabled={deleting}>
          <Button.Text>
            {
              // TRANSLATORS: Button label for confirming removing a device.
              messages.pgettext('device-management', 'Remove')
            }
          </Button.Text>
        </Button>,
        <Button key="back" onClick={hideConfirmDialog} disabled={deleting}>
          <Button.Text>{messages.gettext('Back')}</Button.Text>
        </Button>,
      ]}
      close={hideConfirmDialog}>
      <ModalMessage>
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
      </ModalMessage>
      <Text variant="labelTinySemiBold" color="whiteAlpha60">
        {messages.pgettext(
          'device-management',
          'The device will be removed from the list and logged out.',
        )}
      </Text>
    </ModalAlert>
  );
}
