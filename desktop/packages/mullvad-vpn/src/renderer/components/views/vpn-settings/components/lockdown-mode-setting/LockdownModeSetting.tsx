import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { useScrollToListItem } from '../../../../../hooks';
import { Button } from '../../../../../lib/components';
import { useBoolean } from '../../../../../lib/utility-hooks';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalAlert, ModalAlertType, ModalMessage } from '../../../../Modal';
import { ToggleListItem } from '../../../../toggle-list-item';

export function LockdownModeSetting() {
  const blockWhenDisconnected = useSelector((state) => state.settings.blockWhenDisconnected);
  const { setBlockWhenDisconnected: setBlockWhenDisconnectedImpl } = useAppContext();
  const { ref, animation } = useScrollToListItem('lockdown-mode-setting');

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] =
    useBoolean(false);

  const setBlockWhenDisconnected = useCallback(
    async (blockWhenDisconnected: boolean) => {
      try {
        await setBlockWhenDisconnectedImpl(blockWhenDisconnected);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update block when disconnected', error.message);
      }
    },
    [setBlockWhenDisconnectedImpl],
  );

  const setLockDownMode = useCallback(
    async (newValue: boolean) => {
      if (newValue) {
        showConfirmationDialog();
      } else {
        await setBlockWhenDisconnected(false);
      }
    },
    [setBlockWhenDisconnected, showConfirmationDialog],
  );

  const confirmLockdownMode = useCallback(async () => {
    hideConfirmationDialog();
    await setBlockWhenDisconnected(true);
  }, [hideConfirmationDialog, setBlockWhenDisconnected]);

  return (
    <ToggleListItem
      ref={ref}
      animation={animation}
      checked={blockWhenDisconnected}
      onCheckedChange={setLockDownMode}>
      <ToggleListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Lockdown mode')}
      </ToggleListItem.Label>
      <ToggleListItem.Group>
        <InfoButton>
          <ModalMessage>
            {messages.pgettext(
              'vpn-settings-view',
              'The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents.',
            )}
          </ModalMessage>
          <ModalMessage>
            {messages.pgettext(
              'vpn-settings-view',
              'With Lockdown Mode enabled, you must be connected to a Mullvad VPN server to be able to reach the internet. Manually disconnecting or quitting the app will block your connection.',
            )}
          </ModalMessage>
        </InfoButton>

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
        <ToggleListItem.Switch />
      </ToggleListItem.Group>
    </ToggleListItem>
  );
}
