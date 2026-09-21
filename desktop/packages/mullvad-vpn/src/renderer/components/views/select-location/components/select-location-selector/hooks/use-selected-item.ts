import { LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useSelectedItem() {
  const { locationType } = useSelectLocationViewContext();
  if (locationType === LocationType.entry) {
    return 'entry';
  } else if (locationType === LocationType.entryAutomatic) {
    return 'entryAutomatic';
  } else {
    return 'exit';
  }
}
