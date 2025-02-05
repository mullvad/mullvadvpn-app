import React from 'react';
interface FilterChipContextProps {
  disabled?: boolean;
}
const FilterChipContext = React.createContext<FilterChipContextProps | undefined>(undefined);

export const useFilterChipContext = (): FilterChipContextProps => {
  const context = React.useContext(FilterChipContext);
  if (!context) {
    throw new Error('useFilterChipContext must be used within a FilterChipContext');
  }
  return context;
};

interface FilterChipProviderProps {
  disabled?: boolean;
  children: React.ReactNode;
}

export const FilterChipProvider = ({ disabled, children }: FilterChipProviderProps) => {
  return <FilterChipContext.Provider value={{ disabled }}>{children}</FilterChipContext.Provider>;
};
