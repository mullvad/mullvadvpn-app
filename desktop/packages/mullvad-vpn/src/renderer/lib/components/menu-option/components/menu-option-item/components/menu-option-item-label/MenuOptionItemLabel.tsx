import styled from 'styled-components';

import { BodySmall, type BodySmallProps } from '../../../../../text';
import { useMenuOptionContext } from '../../../../MenuOptionContext';

export type MenuOptionItemLabelProps = BodySmallProps;

export const StyledMenuOptionItemLabel = styled(BodySmall)<{ $disabled?: boolean }>`
  overflow-wrap: anywhere;
`;

export function MenuOptionItemLabel({ children, ...props }: MenuOptionItemLabelProps) {
  const { disabled } = useMenuOptionContext();

  return (
    <StyledMenuOptionItemLabel color={disabled ? 'whiteAlpha20' : 'white'} {...props}>
      {children}
    </StyledMenuOptionItemLabel>
  );
}
