import { LocationType } from '../types';
import { isProvidersFilterActive } from '../utils';
import { useProviders } from './use-providers';

export function useIsProvidersFilterActive(locationType: LocationType) {
  const { providers, activeProviders } = useProviders(locationType);

  if (locationType === LocationType.entryAutomatic) {
    return false;
  }

  return isProvidersFilterActive(providers, activeProviders);
}
