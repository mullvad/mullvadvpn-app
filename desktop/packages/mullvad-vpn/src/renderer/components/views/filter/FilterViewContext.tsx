import React, { useMemo } from 'react';

import { Ownership } from '../../../../shared/daemon-rpc-types';
import { useOwnership, useProviders } from '../../../features/location/hooks';
import { useFilteredProviders } from './hooks';

type FilterViewContextProviderProps = React.PropsWithChildren;

type FilterViewContext = {
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
  const { providers, activeProviders } = useProviders();
  const { activeOwnership } = useOwnership();
  const [selectedProviders, setSelectedProviders] = React.useState<string[]>(activeProviders);
  const [selectedOwnership, setSelectedOwnership] = React.useState<Ownership>(activeOwnership);

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
      selectedProviders,
      toggleProviders,
      availableProviders,
      selectedOwnership,
      setOwnership: setSelectedOwnership,
    }),

    [availableProviders, selectedOwnership, selectedProviders, toggleProviders],
  );

  return <FilterViewContext.Provider value={value}>{children}</FilterViewContext.Provider>;
}
