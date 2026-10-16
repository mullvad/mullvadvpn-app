import { useProviders } from '../../../../features/locations/hooks';
import { useLocationType } from './use-location-type';

export function useActiveProviders() {
  const { exitProviders, entryProviders } = useProviders();
  const locationType = useLocationType();

  const activeProviders = locationType === 'entry' ? entryProviders : exitProviders;

  return activeProviders;
}
