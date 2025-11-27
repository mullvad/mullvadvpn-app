import styled, { css } from 'styled-components';

import { spacings } from '../../foundations';
import { Flex, FlexProps } from '../flex';

type Indent = 'small' | 'medium' | 'large';

export type ContainerProps = FlexProps & {
  indent: Indent;
};

const widths: Record<Indent, string> = {
  small: `calc(100% - ${spacings.small} * 2)`,
  medium: `calc(100% - ${spacings.medium} * 2)`,
  large: `calc(100% - ${spacings.large} * 2)`,
};

export const StyledContainer = styled(Flex)<{ $width: string }>`
  ${({ $width }) => css`
    width: ${$width};
    margin-left: auto;
    margin-right: auto;
  `};
`;

export function Container({ indent, ...props }: ContainerProps) {
  return <StyledContainer $width={widths[indent]} {...props} />;
}
