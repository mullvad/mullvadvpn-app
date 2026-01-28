import { useCallback } from 'react';

import { type RelayLocation, wrapConstraint } from '../../../../../shared/daemon-rpc-types';
import { useRelaySettingsModifier } from '../../../../lib/constraint-updater';
import { LocationType } from '../select-location-types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useOnSelectLocation } from './use-on-select-location';

export function useOnSelectEntryLocation() {
  const onSelectLocation = useOnSelectLocation();
  const { setLocationType } = useSelectLocationViewContext();
  const relaySettingsModifier = useRelaySettingsModifier();

  const onSelectRelay = useCallback(
    async (entryLocation: RelayLocation) => {
      setLocationType(LocationType.exit);
      const settings = relaySettingsModifier((settings) => {
        settings.wireguardConstraints.entryLocation = wrapConstraint(entryLocation);
        return settings;
      });
      await onSelectLocation({ normal: settings });
    },
    [onSelectLocation, relaySettingsModifier, setLocationType],
  );

  const onSelectSpecial = useCallback(
    async (_location: 'any') => {
      setLocationType(LocationType.exit);
      const settings = relaySettingsModifier((settings) => {
        settings.wireguardConstraints.entryLocation = 'any';
        return settings;
      });
      await onSelectLocation({ normal: settings });
    },
    [onSelectLocation, relaySettingsModifier, setLocationType],
  );

  return [onSelectRelay, onSelectSpecial] as const;
}
