import React from 'react';

import { messages } from '../../../../../shared/gettext';
import log from '../../../../../shared/logging';
import { ModalAlert, ModalAlertType, ModalMessage } from '../../../../components/Modal';
import { Button } from '../../../../lib/components';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useBoolean } from '../../../../lib/utility-hooks';
import { useLockdownMode } from '../../hooks';

export type LockdownModeSwitchProp = SwitchProps;

function LockdownModeSwitch({ children, ...props }: LockdownModeSwitchProp) {
  const { lockdownMode, setLockdownMode: setLockdownModeImpl } = useLockdownMode();

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] =
    useBoolean(false);

  const setLockdownMode = React.useCallback(
    async (lockdownMode: boolean) => {
      try {
        await setLockdownModeImpl(lockdownMode);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update lockdown mode', error.message);
      }
    },
    [setLockdownModeImpl],
  );

  const handleOnCheckedChange = React.useCallback(
    async (newValue: boolean) => {
      if (newValue) {
        showConfirmationDialog();
      } else {
        await setLockdownMode(false);
      }
    },
    [setLockdownMode, showConfirmationDialog],
  );

  const confirmLockdownMode = React.useCallback(async () => {
    hideConfirmationDialog();
    await setLockdownMode(true);
  }, [hideConfirmationDialog, setLockdownMode]);

  return (
    <>
      <Switch checked={lockdownMode} onCheckedChange={handleOnCheckedChange} {...props}>
        {children}
      </Switch>
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.caution}
        buttons={[
          <Button variant="destructive" key="confirm" onClick={confirmLockdownMode}>
            <Button.Text>{messages.gettext('Enable anyway')}</Button.Text>
          </Button>,
          <Button key="back" onClick={hideConfirmationDialog}>
            <Button.Text>{messages.gettext('Back')}</Button.Text>
          </Button>,
        ]}
        close={hideConfirmationDialog}>
        <ModalMessage>
          {messages.pgettext(
            'vpn-settings-view',
            'Attention: enabling this will always require a Mullvad VPN connection in order to reach the internet.',
          )}
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'vpn-settings-view',
            'The app’s built-in kill switch is always on. This setting will additionally block the internet if clicking Disconnect or Quit.',
          )}
        </ModalMessage>
      </ModalAlert>
    </>
  );
}

const LockdownModeSwitchNamespace = Object.assign(LockdownModeSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { LockdownModeSwitchNamespace as LockdownModeSwitch };
