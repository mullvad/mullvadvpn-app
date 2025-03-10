import styled from 'styled-components';

import { TitleMedium } from '../../typography';
import { useNavigationHeader } from '../NavigationHeaderContext';

export interface NavigationHeaderTitleProps {
  children: React.ReactNode;
}

export const StyledText = styled(TitleMedium)<{ $visible?: boolean }>(({ $visible = true }) => ({
  opacity: $visible ? 1 : 0,
  transition: 'opacity 250ms ease-in-out',
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
}));

export const NavigationHeaderTitle = ({ children, ...props }: NavigationHeaderTitleProps) => {
  const { titleVisible } = useNavigationHeader();
  return (
    <StyledText forwardedAs="h1" $visible={titleVisible} {...props}>
      {children}
    </StyledText>
  );
};
