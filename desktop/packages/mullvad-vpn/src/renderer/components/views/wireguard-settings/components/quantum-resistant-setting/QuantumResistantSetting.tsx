import { useCallback } from 'react';
import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useScrollToListItem } from '../../../../../hooks';
import { Listbox } from '../../../../../lib/components/listbox/Listbox';
import { useSelector } from '../../../../../redux/store';
import { DefaultListboxOption } from '../../../../default-listbox-option';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';

export function QuantumResistantSetting() {
  const { setWireguardQuantumResistant } = useAppContext();
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);

  const id = 'quantum-resistant-setting';
  const ref = React.useRef<HTMLDivElement>(null);
  const scrollToListItem = useScrollToListItem(ref, id);

  const selectQuantumResistant = useCallback(
    async (quantumResistant: boolean | null) => {
      await setWireguardQuantumResistant(quantumResistant ?? undefined);
    },
    [setWireguardQuantumResistant],
  );

  return (
    <Listbox
      animation={scrollToListItem?.animation}
      value={quantumResistant ?? null}
      onValueChange={selectQuantumResistant}>
      <Listbox.Item ref={ref}>
        <Listbox.Content>
          <Listbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard quantum resistance selector. This setting
              // TRANSLATORS: makes the cryptography resistant to the future abilities of quantum
              // TRANSLATORS: computers.
              messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel')
            }
          </Listbox.Label>
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
        </Listbox.Content>
      </Listbox.Item>
      <Listbox.Options>
        <DefaultListboxOption value={null}>{messages.gettext('Automatic')}</DefaultListboxOption>
        <DefaultListboxOption value={true}>{messages.gettext('On')}</DefaultListboxOption>
        <DefaultListboxOption value={false}>{messages.gettext('Off')}</DefaultListboxOption>
      </Listbox.Options>
    </Listbox>
  );
}
