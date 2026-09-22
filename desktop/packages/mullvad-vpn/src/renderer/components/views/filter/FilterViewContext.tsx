import React, { useMemo } from 'react';

import { Ownership } from '../../../../shared/daemon-rpc-types';
import { useOwnership, useProviders } from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import { useHistory } from '../../../lib/history';
import { useFilteredProviders } from './hooks';

type FilterViewContextProviderProps = React.PropsWithChildren;

type FilterViewContext = {
  locationType: LocationType;
  selectedProviders: string[];
  availableProviders: string[];
  toggleProviders: (providers: string[]) => void;
  selectedOwnership: Ownership;
  setOwnership: React.Dispatch<React.SetStateAction<Ownership>>;
};

const FilterViewContext = React.createContext<FilterViewContext | undefined>(undefined);

export const useFilterViewContext = (): FilterViewContext => {
  const context = React.useContext(FilterViewContext);
  if (!context) {
    throw new Error('useFilterViewContext must be used within a FilterViewContext');
  }
  return context;
};

export function FilterViewContextProvider({ children }: FilterViewContextProviderProps) {
  const history = useHistory();

  const { location } = history;
  const { state } = location;

  const filterViewOption = state.options?.find(
    (option) => option.type === 'filter-view-location-type',
  );
  const locationType =
    filterViewOption?.locationType === 'entry' ? LocationType.entry : LocationType.exit;

  const { providers, activeProviders } = useProviders(locationType);
  const { ownership } = useOwnership(locationType);
  const [selectedProviders, setSelectedProviders] = React.useState<string[]>(activeProviders);
  const [selectedOwnership, setSelectedOwnership] = React.useState<Ownership>(ownership);

  const availableProviders = useFilteredProviders(providers, selectedOwnership);

  const toggleProviders = React.useCallback(
    (nextProviders: string[]) => {
      setSelectedProviders((currentSelectedProviders) => {
        const allSelected = availableProviders.every((provider) =>
          currentSelectedProviders.includes(provider),
        );
        const selectingAll = availableProviders.every((provider) =>
          nextProviders.includes(provider),
        );
        if (allSelected && selectingAll) {
          return [];
        } else {
          return nextProviders;
        }
      });
    },
    [availableProviders],
  );

  const value = useMemo(
    () => ({
      locationType,
      selectedProviders,
      toggleProviders,
      availableProviders,
      selectedOwnership,
      setOwnership: setSelectedOwnership,
    }),

    [availableProviders, locationType, selectedOwnership, selectedProviders, toggleProviders],
  );

  return <FilterViewContext.Provider value={value}>{children}</FilterViewContext.Provider>;
}
