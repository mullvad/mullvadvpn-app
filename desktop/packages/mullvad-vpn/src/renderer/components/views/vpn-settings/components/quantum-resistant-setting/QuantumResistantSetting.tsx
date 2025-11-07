import { messages } from '../../../../../../shared/gettext';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListItem } from '../../../../settings-list-item';
import { QuantumResistantSwitch } from './QuantumResistantSwitch';

export function QuantumResistantSetting() {
  return (
    <SettingsListItem anchorId="quantum-resistant-setting">
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <QuantumResistantSwitch>
            <QuantumResistantSwitch.Label variant="titleMedium">
              {
                // TRANSLATORS: The title for the WireGuard quantum resistance selector. This setting
                // TRANSLATORS: makes the cryptography resistant to the future abilities of quantum
                // TRANSLATORS: computers.
                messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel')
              }
            </QuantumResistantSwitch.Label>
            <SettingsListItem.Group>
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

              <QuantumResistantSwitch.Trigger>
                <QuantumResistantSwitch.Thumb />
              </QuantumResistantSwitch.Trigger>
            </SettingsListItem.Group>
          </QuantumResistantSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
