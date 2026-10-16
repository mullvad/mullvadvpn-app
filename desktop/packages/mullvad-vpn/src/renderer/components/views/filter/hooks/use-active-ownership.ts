import { useOwnership } from '../../../../features/locations/hooks';
import { useLocationType } from './use-location-type';

export function useActiveOwnership() {
  const { exitOwnership, entryOwnership } = useOwnership();
  const locationType = useLocationType();

  const activeOwnership = locationType === 'entry' ? entryOwnership : exitOwnership;

  return activeOwnership;
}
