import React from 'react';

import { IconButtonProps } from './IconButton';
interface IconButtonContextProps {
  disabled?: boolean;
  variant: IconButtonProps['variant'];
  size: IconButtonProps['size'];
}

const IconButtonContext = React.createContext<IconButtonContextProps | undefined>(undefined);

export const useIconButtonContext = (): IconButtonContextProps => {
  const context = React.useContext(IconButtonContext);
  if (!context) {
    throw new Error('useButtonContext must be used within a IconButtonProvider');
  }
  return context;
};
interface IconButtonProviderProps extends IconButtonContextProps {
  children: React.ReactNode;
}
export const IconButtonProvider = ({ children, ...props }: IconButtonProviderProps) => {
  return <IconButtonContext.Provider value={props}>{children}</IconButtonContext.Provider>;
};
