import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { Flex, type FlexProps } from '../../../flex';

export type MenuDividerProps = FlexProps;

export const StyledMenuDivider = styled(Flex)<MenuDividerProps>`
  height: 1px;
  background-color: ${colors.darkBlue};
`;

export function MenuDivider(props: MenuDividerProps) {
  return <StyledMenuDivider margin={{ top: 'tiny' }} {...props} />;
}
