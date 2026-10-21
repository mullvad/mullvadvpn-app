import { LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useLocationInvalidReason(locationType: LocationType) {
  const { invalidReason } = useSelectLocationViewContext();

  if (locationType === LocationType.entry && invalidReason.entry) {
    return invalidReason.entry;
  } else if (locationType === LocationType.exit && invalidReason.exit) {
    return invalidReason.exit;
  }

  return undefined;
}
