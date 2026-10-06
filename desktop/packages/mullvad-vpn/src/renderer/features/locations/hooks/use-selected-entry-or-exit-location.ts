import type { LiftedConstraint, RelayLocation } from '../../../../shared/daemon-rpc-types';
import { LocationType } from '../types';
import { useEntryLocation } from './use-entry-location';
import { useExitLocation } from './use-exit-location';

export function useSelectedEntryOrExitLocation(
  locationType: LocationType,
): LiftedConstraint<RelayLocation> {
  const { entryLocation } = useEntryLocation();
  const { exitLocation } = useExitLocation();
  return locationType === 'entry' ? entryLocation : exitLocation;
}
