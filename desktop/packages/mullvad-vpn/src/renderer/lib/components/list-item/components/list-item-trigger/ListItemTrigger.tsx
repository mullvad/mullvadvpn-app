import styled, { css } from 'styled-components';

import { Colors } from '../../../../foundations';
import { ButtonBase } from '../../../button';
import { useListItem } from '../../ListItemContext';
import { StyledFlex } from '../list-item-content';

// TODO: Colors should be replace with
// with new color tokens once they are implemented.
const StyledButton = styled(ButtonBase)<{ $disabled?: boolean }>`
  display: flex;
  width: 100%;
  ${({ $disabled }) =>
    !$disabled &&
    css`
      &:hover ${StyledFlex} {
        background-color: rgba(56, 86, 116, 1);
      }
      &:active ${StyledFlex} {
        background-color: rgba(62, 95, 129, 1);
      }
    `}

  &&:focus-visible {
    outline: 2px solid ${Colors.white};
    outline-offset: -1px;
    z-index: 10;
  }
`;

export type ListItemTriggerProps = React.HtmlHTMLAttributes<HTMLButtonElement>;

export const ListItemTrigger = ({ children, ...props }: ListItemTriggerProps) => {
  const { disabled } = useListItem();
  return (
    <StyledButton $disabled={disabled} {...props}>
      {children}
    </StyledButton>
  );
};
