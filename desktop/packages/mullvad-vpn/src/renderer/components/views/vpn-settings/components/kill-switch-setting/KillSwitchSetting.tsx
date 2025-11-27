import { messages } from '../../../../../../shared/gettext';
import { Switch } from '../../../../../lib/components/switch';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListItem } from '../../../../settings-list-item';

export function KillSwitchSetting() {
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <SettingsListItem.Label>
            {messages.pgettext('vpn-settings-view', 'Kill switch')}
          </SettingsListItem.Label>
          <SettingsListItem.Group gap="medium">
            <InfoButton>
              <ModalMessage>
                {messages.pgettext(
                  'vpn-settings-view',
                  'This built-in feature prevents your traffic from leaking outside of the VPN tunnel if your network suddenly stops working or if the tunnel fails, it does this by blocking your traffic until your connection is reestablished.',
                )}
              </ModalMessage>
              <ModalMessage>
                {messages.pgettext(
                  'vpn-settings-view',
                  'The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents. With Lockdown Mode enabled, you must be connected to a Mullvad VPN server to be able to reach the internet. Manually disconnecting or quitting the app will block your connection.',
                )}
              </ModalMessage>
            </InfoButton>
            <Switch checked disabled>
              <Switch.Trigger>
                <Switch.Thumb />
              </Switch.Trigger>
            </Switch>
          </SettingsListItem.Group>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
