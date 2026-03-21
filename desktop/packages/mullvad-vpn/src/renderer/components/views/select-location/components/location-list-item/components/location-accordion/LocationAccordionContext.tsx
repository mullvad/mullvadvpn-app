import React from 'react';

type LocationAccordionContextProps = Omit<LocationAccordionProviderProps, 'children'> & {
  userTriggeredExpand: boolean;
  setUserTriggeredExpand: React.Dispatch<React.SetStateAction<boolean>>;
};

const LocationAccordionContext = React.createContext<LocationAccordionContextProps | undefined>(
  undefined,
);

export const useLocationAccordionContext = (): LocationAccordionContextProps => {
  const context = React.useContext(LocationAccordionContext);
  if (!context) {
    throw new Error('useLocationAccordionContext must be used within a LocationAccordionProvider');
  }
  return context;
};

type LocationAccordionProviderProps = React.PropsWithChildren;

export function LocationAccordionProvider({ children }: LocationAccordionProviderProps) {
  const [userTriggeredExpand, setUserTriggeredExpand] = React.useState(false);
  const value = React.useMemo(
    () => ({ userTriggeredExpand, setUserTriggeredExpand }),
    [userTriggeredExpand, setUserTriggeredExpand],
  );
  return (
    <LocationAccordionContext.Provider value={value}>{children}</LocationAccordionContext.Provider>
  );
}
