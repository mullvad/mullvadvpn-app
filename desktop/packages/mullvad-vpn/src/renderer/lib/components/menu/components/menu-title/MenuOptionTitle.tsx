import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { type BodySmallProps, LabelTinySemiBold } from '../../../text';

export type MenuTitleProps = BodySmallProps;

export const StyledMenuTitle = styled(LabelTinySemiBold)`
  display: inline-block;
  padding: ${spacings.tiny} ${spacings.small};
`;

export function MenuTitle({ children, ...props }: MenuTitleProps) {
  return (
    <StyledMenuTitle color="white" {...props}>
      {children}
    </StyledMenuTitle>
  );
}
