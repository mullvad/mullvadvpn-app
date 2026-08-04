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
  const filterViewOptions = history.location.state.options?.find(
    (option) => option.type === 'filter-view-options',
  );
  const locationType = filterViewOptions?.locationType ?? LocationType.exit;

  const { exitOwnership, entryOwnership } = useOwnership();
  const activeOwnership = locationType === LocationType.entry ? entryOwnership : exitOwnership;
  const [selectedOwnership, setSelectedOwnership] = React.useState<Ownership>(activeOwnership);

  const { providers, exitProviders, entryProviders } = useProviders();
  const activeProviders = locationType === LocationType.entry ? entryProviders : exitProviders;
  const [selectedProviders, setSelectedProviders] = React.useState<string[]>(activeProviders);

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
