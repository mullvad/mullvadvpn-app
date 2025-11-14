import styled, { css } from 'styled-components';

import { spacings } from '../../foundations';
import { Flex, FlexProps } from '../flex';

type MarginInline = 'small' | 'medium' | 'large';

export type ContainerProps = FlexProps & {
  marginInline: MarginInline;
};

export const StyledContainer = styled(Flex)<{ $marginInline: string }>`
  ${({ $marginInline }) => css`
    margin-inline: ${$marginInline};
  `}
`;

export function Container({ marginInline, ...props }: ContainerProps) {
  const spacing = spacings[marginInline];
  return <StyledContainer $marginInline={spacing} {...props} />;
}
