import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../foundations';
import { Flex, FlexProps } from '../flex';

export interface ContainerProps extends FlexProps {
  size?: '3' | '4';
  children: React.ReactNode;
}

const sizes: Record<'3' | '4', string> = {
  '3': `calc(100% - ${spacings.large} * 2)`,
  '4': `calc(100% - ${spacings.medium} * 2)`,
};

const StyledFlex = styled(Flex)<{ $size: string }>((props) => ({
  width: props.$size,
  margin: 'auto',
}));

export const Container = React.forwardRef<HTMLDivElement, ContainerProps>(
  ({ size = '4', ...props }, ref) => {
    return <StyledFlex ref={ref} $size={sizes[size]} {...props} />;
  },
);

Container.displayName = 'Container';
