import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { useMenuOptionContext } from '../../MenuOptionContext';
import { StyledMenuOptionItem } from '../menu-option-item/MenuOptionItem';

export type MenuOptionTriggerProps = React.HtmlHTMLAttributes<HTMLButtonElement>;

export const StyledListItemTrigger = styled.button`
  display: flex;
  background-color: transparent;

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

export function MenuOptionTrigger(props: MenuOptionTriggerProps) {
  const { disabled } = useMenuOptionContext();
  return <StyledListItemTrigger disabled={disabled} {...props} />;
}
