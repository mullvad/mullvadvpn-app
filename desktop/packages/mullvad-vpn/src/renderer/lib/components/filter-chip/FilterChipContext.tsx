import React from 'react';
type FilterChipContextProps = Omit<FilterChipProviderProps, 'children'> & {
  labelId: string;
};

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
  children: React.ReactNode;
};

export const FilterChipProvider = ({ disabled, children }: FilterChipProviderProps) => {
  const labelId = React.useId();
  return (
    <FilterChipContext.Provider value={{ labelId, disabled }}>
      {children}
    </FilterChipContext.Provider>
  );
};
