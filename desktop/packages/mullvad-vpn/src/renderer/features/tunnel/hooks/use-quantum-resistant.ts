import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useQuantumResistant() {
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);
  const { setWireguardQuantumResistant: contextSetQuantumResistance } = useAppContext();
  const setQuantumResistant = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetQuantumResistance(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set quantum resistant', message);
      }
    },
    [contextSetQuantumResistance],
  );

  return { quantumResistant, setQuantumResistant };
}
