import React from 'react';
import styled from 'styled-components';

import { Spacings } from '../../foundations';
import { FlexProps, Flex } from '../flex';

export interface ContainerProps extends FlexProps {
  size?: '3' | '4';
  children: React.ReactNode;
}

const sizes: Record<'3' | '4', string> = {
  '3': `calc(100% - ${Spacings.spacing6} * 2)`,
  '4': `calc(100% - ${Spacings.spacing5} * 2)`,
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
