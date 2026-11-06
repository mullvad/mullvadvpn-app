import { LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useLocationInvalidReason(locationType: LocationType) {
  const { invalidReason } = useSelectLocationViewContext();

  if (locationType === 'entry' && invalidReason.entry) {
    return invalidReason.entry;
  } else if (locationType === 'exit' && invalidReason.exit) {
    return invalidReason.exit;
  }

  return undefined;
}
