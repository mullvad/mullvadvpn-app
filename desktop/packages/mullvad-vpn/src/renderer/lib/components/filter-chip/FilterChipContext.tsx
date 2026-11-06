import React from 'react';
type FilterChipContextProps = Omit<FilterChipProviderProps, 'children'>;

const FilterChipContext = React.createContext<FilterChipContextProps | undefined>(undefined);

export const useFilterChipContext = (): FilterChipContextProps => {
  const context = React.useContext(FilterChipContext);
  if (!context) {
    throw new Error('useFilterChipContext must be used within a FilterChipContext');
  }
  return context;
};

type FilterChipProviderProps = {
  disabled?: boolean;
  inactive?: boolean;
  children: React.ReactNode;
};

export const FilterChipProvider = ({ disabled, inactive, children }: FilterChipProviderProps) => {
  return (
    <FilterChipContext.Provider value={{ disabled, inactive }}>
      {children}
    </FilterChipContext.Provider>
  );
};
