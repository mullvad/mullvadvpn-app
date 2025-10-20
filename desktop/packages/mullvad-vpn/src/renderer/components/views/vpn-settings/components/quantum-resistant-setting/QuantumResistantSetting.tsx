import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListbox } from '../../../../settings-listbox';

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
    <SettingsListbox
      anchorId="quantum-resistant-setting"
      value={quantumResistant}
      onValueChange={selectQuantumResistant}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard quantum resistance selector. This setting
              // TRANSLATORS: makes the cryptography resistant to the future abilities of quantum
              // TRANSLATORS: computers.
              messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel')
            }
          </SettingsListbox.Label>
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
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <SettingsListbox.BaseOption value={true}>
          {messages.gettext('On')}
        </SettingsListbox.BaseOption>
        <SettingsListbox.BaseOption value={false}>
          {messages.gettext('Off')}
        </SettingsListbox.BaseOption>
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}
