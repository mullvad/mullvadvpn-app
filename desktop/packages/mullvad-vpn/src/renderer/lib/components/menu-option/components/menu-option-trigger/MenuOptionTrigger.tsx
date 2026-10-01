import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { Trigger, type TriggerProps } from '../../../trigger';
import { useMenuOptionContext } from '../../MenuOptionContext';
import { StyledMenuOptionItem } from '../menu-option-item/MenuOptionItem';

export type MenuOptionTriggerProps<T extends React.ElementType = 'button'> = TriggerProps<T>;

export const StyledListItemTrigger = styled(Trigger)`
  display: flex;
  background-color: transparent;
  width: 100%;

  &:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
  }

  &:not(:disabled):hover {
    ${StyledMenuOptionItem} {
      background-color: ${colors.blue};
    }
  }

  &:not(:disabled):active {
    ${StyledMenuOptionItem} {
      background-color: ${colors.whiteOnBlue10};
    }
  }
`;

export function MenuOptionTrigger<T extends React.ElementType = 'button'>({
  as,
  ...props
}: MenuOptionTriggerProps<T>) {
  const { disabled } = useMenuOptionContext();
  return <StyledListItemTrigger forwardedAs={as} disabled={disabled} {...props} />;
}
