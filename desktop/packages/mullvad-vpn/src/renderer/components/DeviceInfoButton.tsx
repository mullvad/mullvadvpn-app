import { messages } from '../../shared/gettext';
import { Info } from './info';

export default function DeviceInfoButton() {
  return (
    <Info>
      <Info.Button size="small" />
      <Info.Dialog>
        <Info.Dialog.Text>
          {messages.pgettext(
            'device-management',
            'This is the name assigned to the device. Each device logged in on a Mullvad account gets a unique name that helps you identify it when you manage your devices in the app or on the website.',
          )}
        </Info.Dialog.Text>
        <Info.Dialog.Text>
          {messages.pgettext(
            'device-management',
            'You can have up to 5 devices logged in on one Mullvad account.',
          )}
        </Info.Dialog.Text>
        <Info.Dialog.Text>
          {messages.pgettext(
            'device-management',
            'If you log out, the device and the device name is removed. When you log back in again, the device will get a new name.',
          )}
        </Info.Dialog.Text>
      </Info.Dialog>
    </Info>
  );
}
