import { messages } from '../../../../../shared/gettext';
import InfoButton from '../../../../components/InfoButton';
import { ModalMessage } from '../../../../components/Modal';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { LockdownModeSwitch } from '../lockdown-mode-switch';

export function LockdownModeSetting() {
  return (
    <SettingsListItem anchorId="lockdown-mode-setting">
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <LockdownModeSwitch>
            <LockdownModeSwitch.Label variant="titleMedium">
              {messages.pgettext('vpn-settings-view', 'Lockdown mode')}
            </LockdownModeSwitch.Label>
            <SettingsListItem.Group>
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
              <LockdownModeSwitch.Trigger>
                <LockdownModeSwitch.Thumb />
              </LockdownModeSwitch.Trigger>
            </SettingsListItem.Group>
          </LockdownModeSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
