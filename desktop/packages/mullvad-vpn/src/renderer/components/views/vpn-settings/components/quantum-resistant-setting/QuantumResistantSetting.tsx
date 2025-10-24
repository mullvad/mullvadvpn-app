import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function QuantumResistantSetting() {
  const { setWireguardQuantumResistant } = useAppContext();
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);

  const selectQuantumResistant = useCallback(
    async (quantumResistant: boolean) => {
      await setWireguardQuantumResistant(quantumResistant);
    },
    [setWireguardQuantumResistant],
  );

  return (
    <SettingsToggleListItem
      anchorId="quantum-resistant-setting"
      checked={quantumResistant}
      onCheckedChange={selectQuantumResistant}>
      <SettingsToggleListItem.Label>
        {
          // TRANSLATORS: The title for the WireGuard quantum resistance selector. This setting
          // TRANSLATORS: makes the cryptography resistant to the future abilities of quantum
          // TRANSLATORS: computers.
          messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel')
        }
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Group>
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
        <SettingsToggleListItem.Switch />
      </SettingsToggleListItem.Group>
    </SettingsToggleListItem>
  );
}
