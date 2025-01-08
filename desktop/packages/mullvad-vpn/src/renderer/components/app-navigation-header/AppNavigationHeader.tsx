import { useContext } from 'react';

import { NavigationHeader, NavigationHeaderProps } from '../../lib/components';
import { NavigationScrollContext } from '../NavigationContainer';
import { AppNavigationHeaderBackButton, AppNavigationHeaderInfoButton } from './components';

export interface NavigationBarProps extends NavigationHeaderProps {
  title?: string;
  children?: React.ReactNode;
}

const AppNavigationHeader = ({ title, children, ...props }: NavigationBarProps) => {
  const { showsBarTitle } = useContext(NavigationScrollContext);
  return (
    <NavigationHeader titleVisible={showsBarTitle} {...props}>
      <AppNavigationHeaderBackButton />
      {title && <NavigationHeader.Title>{title}</NavigationHeader.Title>}
      <NavigationHeader.ButtonGroup $justifyContent="flex-end">
        {children}
      </NavigationHeader.ButtonGroup>
    </NavigationHeader>
  );
};

const AppNavigationHeaderNamespace = Object.assign(AppNavigationHeader, {
  IconButton: NavigationHeader.IconButton,
  InfoButton: AppNavigationHeaderInfoButton,
});

export { AppNavigationHeaderNamespace as AppNavigationHeader };
