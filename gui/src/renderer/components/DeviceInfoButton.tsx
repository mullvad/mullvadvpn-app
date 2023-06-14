import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { useBoolean } from '../lib/utilityHooks';
import * as AppButton from './AppButton';
import { InfoIcon } from './InfoButton';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';

const StyledInfoButton = styled.button({
  margin: '0 0 0 10px',
  borderWidth: 0,
  padding: 0,
  cursor: 'default',
  backgroundColor: 'transparent',
});

export default function DeviceInfoButton() {
  const [deviceHelpVisible, showDeviceHelp, hideDeviceHelp] = useBoolean();

  return (
    <>
      <StyledInfoButton
        onClick={showDeviceHelp}
        aria-label={messages.pgettext('accessibility', 'More information')}>
        <InfoIcon size={16} />
      </StyledInfoButton>
      <ModalAlert
        isOpen={deviceHelpVisible}
        type={ModalAlertType.info}
        buttons={[
          <AppButton.BlueButton key="back" onClick={hideDeviceHelp}>
            {messages.gettext('Got it!')}
          </AppButton.BlueButton>,
        ]}
        close={hideDeviceHelp}>
        <ModalMessage>
          {messages.pgettext(
            'device-management',
            'This is the name assigned to the device. Each device logged in on a Mullvad account gets a unique name that helps you identify it when you manage your devices in the app or on the website.',
          )}
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'device-management',
            'You can have up to 5 devices logged in on one Mullvad account.',
          )}
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'device-management',
            'If you log out, the device and the device name is removed. When you log back in again, the device will get a new name.',
          )}
        </ModalMessage>
      </ModalAlert>
    </>
  );
}
