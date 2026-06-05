import type { RelayLocation } from '../../../../shared/daemon-rpc-types';
import { LocationType } from '../types';
import { useSelectedLocations } from './use-selected-locations';

export function useSelectedEntryOrExitLocation(
  locationType: LocationType,
): RelayLocation | undefined {
  const selectedLocations = useSelectedLocations();
  return locationType === LocationType.entry ? selectedLocations.entry : selectedLocations.exit;
}
