import { messages } from '../../../../../shared/gettext';
import { Info } from '../../../../components/info';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { QuantumResistantSwitch } from '../quantum-resistant-switch';

export type QuantumResistantSettingProps = Omit<ListItemProps, 'children'>;

export function QuantumResistantSetting(props: QuantumResistantSettingProps) {
  return (
    <SettingsListItem anchorId="quantum-resistant-setting" {...props}>
      <SettingsListItem.Item>
        <QuantumResistantSwitch>
          <QuantumResistantSwitch.Label>
            {
              // TRANSLATORS: The title for the WireGuard quantum resistance selector. This setting
              // TRANSLATORS: makes the cryptography resistant to the future abilities of quantum
              // TRANSLATORS: computers.
              messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel')
            }
          </QuantumResistantSwitch.Label>
          <SettingsListItem.Item.ActionGroup>
            <Info>
              <Info.Button />
              <Info.Dialog>
                <Info.Dialog.Text>
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.',
                  )}
                </Info.Dialog.Text>
                <Info.Dialog.Text>
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'It does this by performing an extra key exchange using a quantum safe algorithm and mixing the result into WireGuard’s regular encryption.',
                  )}
                </Info.Dialog.Text>
              </Info.Dialog>
            </Info>

            <QuantumResistantSwitch.Input />
          </SettingsListItem.Item.ActionGroup>
        </QuantumResistantSwitch>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
