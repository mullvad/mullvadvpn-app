import React from 'react';

type LocationListItemAccordionContextProps = Omit<
  LocationListItemAccordionProviderProps,
  'children'
> & {
  userTriggeredExpand: boolean;
  setUserTriggeredExpand: React.Dispatch<React.SetStateAction<boolean>>;
};

const LocationListItemAccordionContext = React.createContext<
  LocationListItemAccordionContextProps | undefined
>(undefined);

export const useLocationListItemAccordionContext = (): LocationListItemAccordionContextProps => {
  const context = React.useContext(LocationListItemAccordionContext);
  if (!context) {
    throw new Error(
      'useLocationListItemAccordionContext must be used within a LocationListItemAccordionProvider',
    );
  }
  return context;
};

type LocationListItemAccordionProviderProps = React.PropsWithChildren;

export function LocationListItemAccordionProvider({
  children,
}: LocationListItemAccordionProviderProps) {
  const [userTriggeredExpand, setUserTriggeredExpand] = React.useState(false);
  const value = React.useMemo(
    () => ({ userTriggeredExpand, setUserTriggeredExpand }),
    [userTriggeredExpand, setUserTriggeredExpand],
  );
  return (
    <LocationListItemAccordionContext.Provider value={value}>
      {children}
    </LocationListItemAccordionContext.Provider>
  );
}
