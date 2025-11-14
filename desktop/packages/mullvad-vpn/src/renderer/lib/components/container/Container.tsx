import styled from 'styled-components';

import { spacings } from '../../foundations';
import { Flex, FlexProps } from '../flex';

export type ContainerProps = FlexProps & {
  size: '3' | '4';
};

const sizes: Record<'3' | '4', string> = {
  '3': `calc(100% - ${spacings.large} * 2)`,
  '4': `calc(100% - ${spacings.medium} * 2)`,
};

export const StyledContainer = styled(Flex)<{ $size: string }>`
  ${({ $size }) => ({
    width: $size,
    margin: 'auto',
  })}
`;

export function Container({ size = '4', ...props }: ContainerProps) {
  return <StyledContainer $size={sizes[size]} {...props} />;
}
