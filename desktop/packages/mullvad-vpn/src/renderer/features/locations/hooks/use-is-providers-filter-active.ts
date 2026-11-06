import { LocationType } from '../types';
import { isProvidersFilterActive } from '../utils';
import { useProviders } from './use-providers';

export function useIsProvidersFilterActive(locationType: LocationType) {
  const { providers, entryProviders, exitProviders } = useProviders();
  const activeProviders = locationType === 'exit' ? exitProviders : entryProviders;

  return isProvidersFilterActive(providers, activeProviders);
}
