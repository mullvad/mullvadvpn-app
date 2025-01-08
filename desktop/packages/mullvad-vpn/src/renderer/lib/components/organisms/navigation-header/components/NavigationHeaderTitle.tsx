import styled from 'styled-components';

import { TitleMedium } from '../../../typography';
import { useNavigationHeader } from '../NavigationHeaderContext';

export interface NavigationHeaderTitleProps {
  children: React.ReactNode;
}

export const StyledText = styled(TitleMedium)<{ $visible?: boolean }>(({ $visible = true }) => ({
  opacity: $visible ? 1 : 0,
  transition: 'opacity 250ms ease-in-out',
}));

export const NavigationHeaderTitle = ({ children }: NavigationHeaderTitleProps) => {
  const { titleVisible } = useNavigationHeader();
  return (
    <StyledText tag="h1" $visible={titleVisible}>
      {children}
    </StyledText>
  );
};
