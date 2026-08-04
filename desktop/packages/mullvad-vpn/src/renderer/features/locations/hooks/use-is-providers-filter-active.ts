import { LocationType } from '../types';
import { useProviders } from './use-providers';

export function useIsProvidersFilterActive(locationType: LocationType) {
  const { providers, entryProviders, exitProviders } = useProviders();
  const activeProviders = locationType === LocationType.entry ? entryProviders : exitProviders;

  const isProvidersFilterActive = activeProviders.length !== providers.length;

  return isProvidersFilterActive;
}
