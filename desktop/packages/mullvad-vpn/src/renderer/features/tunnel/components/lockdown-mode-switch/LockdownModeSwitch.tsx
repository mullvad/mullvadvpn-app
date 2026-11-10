import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { StatusDialog } from '../../../../components/status-dialog';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useLockdownMode } from '../../hooks';

export type LockdownModeSwitchProp = SwitchProps;

function LockdownModeSwitch({ children, ...props }: LockdownModeSwitchProp) {
  const { lockdownMode, setLockdownMode } = useLockdownMode();

  const [confirmDialogOpen, setConfirmDialogOpen] = React.useState(false);

  const handleOnCheckedChange = React.useCallback(
    async (newValue: boolean) => {
      if (newValue) {
        setConfirmDialogOpen(true);
      } else {
        await setLockdownMode(false);
      }
    },
    [setLockdownMode],
  );

  const confirmLockdownMode = React.useCallback(async () => {
    setConfirmDialogOpen(false);
    await setLockdownMode(true);
  }, [setLockdownMode]);

  return (
    <>
      <Switch checked={lockdownMode} onCheckedChange={handleOnCheckedChange} {...props}>
        {children}
      </Switch>
      <StatusDialog variant="info" open={confirmDialogOpen} onOpenChange={setConfirmDialogOpen}>
        <StatusDialog.Text>
          {
            // TRANSLATORS: Warning message shown in dialog when enabling lockdown mode.
            messages.pgettext(
              'vpn-settings-view',
              'Warning: this will block the internet if you aren’t connected through the Mullvad app.',
            )
          }
        </StatusDialog.Text>
        <StatusDialog.ButtonGroup>
          <StatusDialog.Button color="destructive" onClick={confirmLockdownMode}>
            <StatusDialog.Button.Text>{messages.gettext('Enable')}</StatusDialog.Button.Text>
          </StatusDialog.Button>
          <StatusDialog.CloseButton>
            <StatusDialog.CloseButton.Text>
              {messages.gettext('Cancel')}
            </StatusDialog.CloseButton.Text>
          </StatusDialog.CloseButton>
        </StatusDialog.ButtonGroup>
      </StatusDialog>
    </>
  );
}

const LockdownModeSwitchNamespace = Object.assign(LockdownModeSwitch, {
  Label: Switch.Label,
  Input: Switch.Input,
  Trigger: Switch.Trigger,
});

export { LockdownModeSwitchNamespace as LockdownModeSwitch };
