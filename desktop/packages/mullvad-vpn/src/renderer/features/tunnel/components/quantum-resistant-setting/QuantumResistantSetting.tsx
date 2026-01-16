import { messages } from '../../../../../shared/gettext';
import InfoButton from '../../../../components/InfoButton';
import { ModalMessage } from '../../../../components/Modal';
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
          <SettingsListItem.ActionGroup>
            <InfoButton>
              <>
                <ModalMessage>
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.',
                  )}
                </ModalMessage>
                <ModalMessage>
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'It does this by performing an extra key exchange using a quantum safe algorithm and mixing the result into WireGuard’s regular encryption. This extra step uses approximately 500 kiB of traffic every time a new tunnel is established.',
                  )}
                </ModalMessage>
              </>
            </InfoButton>

            <QuantumResistantSwitch.Thumb />
          </SettingsListItem.ActionGroup>
        </QuantumResistantSwitch>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
