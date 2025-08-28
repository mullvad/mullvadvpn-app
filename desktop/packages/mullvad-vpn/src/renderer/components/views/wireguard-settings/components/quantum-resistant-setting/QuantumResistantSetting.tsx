import { useCallback, useMemo } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaInputGroup } from '../../../../AriaGroup';
import Selector, { SelectorItem } from '../../../../cell/Selector';
import { ModalMessage } from '../../../../Modal';

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function QuantumResistantSetting() {
  const { setWireguardQuantumResistant } = useAppContext();
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);

  const items: SelectorItem<boolean>[] = useMemo(
    () => [
      {
        label: messages.gettext('On'),
        value: true,
      },
      {
        label: messages.gettext('Off'),
        value: false,
      },
    ],
    [],
  );

  const selectQuantumResistant = useCallback(
    async (quantumResistant: boolean | null) => {
      await setWireguardQuantumResistant(quantumResistant ?? undefined);
    },
    [setWireguardQuantumResistant],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          title={
            // TRANSLATORS: The title for the WireGuard quantum resistance selector. This setting
            // TRANSLATORS: makes the cryptography resistant to the future abilities of quantum
            // TRANSLATORS: computers.
            messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel')
          }
          details={
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
          }
          items={items}
          value={quantumResistant ?? null}
          onSelect={selectQuantumResistant}
          automaticValue={null}
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}
