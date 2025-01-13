import { createContext, useContext } from 'react';

interface NavigationHeaderContextProps {
  titleVisible: boolean;
}

const NavigationHeaderContext = createContext<NavigationHeaderContextProps | undefined>(undefined);

export const NavigationHeaderProvider = ({
  titleVisible,
  children,
}: React.PropsWithChildren<NavigationHeaderContextProps>) => (
  <NavigationHeaderContext.Provider value={{ titleVisible }}>
    {children}
  </NavigationHeaderContext.Provider>
);

export const useNavigationHeader = (): NavigationHeaderContextProps => {
  const context = useContext(NavigationHeaderContext);
  if (context === undefined) {
    throw new Error('useNavigationHeader must be used within a NavigationHeaderProvider');
  }
  return context;
};
