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
          {messages.pgettext(
            'vpn-settings-view',
            'Attention: enabling this will always require a Mullvad VPN connection in order to reach the internet.',
          )}
        </StatusDialog.Text>
        <StatusDialog.Text>
          {messages.pgettext(
            'vpn-settings-view',
            'The app’s built-in kill switch is always on. This setting will additionally block the internet if clicking Disconnect or Quit.',
          )}
        </StatusDialog.Text>
        <StatusDialog.ButtonGroup>
          <StatusDialog.Button variant="destructive" onClick={confirmLockdownMode}>
            <StatusDialog.Button.Text>{messages.gettext('Enable anyway')}</StatusDialog.Button.Text>
          </StatusDialog.Button>
          <StatusDialog.CloseButton>
            <StatusDialog.CloseButton.Text>
              {messages.gettext('Back')}
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
