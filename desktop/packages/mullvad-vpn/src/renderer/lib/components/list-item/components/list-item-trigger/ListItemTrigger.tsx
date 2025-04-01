import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { ButtonBase } from '../../../button';
import { useListItem } from '../../ListItemContext';
import { StyledFlex } from '../list-item-content';

const StyledButton = styled(ButtonBase)<{ $disabled?: boolean }>`
  display: flex;
  width: 100%;
  ${({ $disabled }) =>
    !$disabled &&
    css`
      &:hover ${StyledFlex} {
        background-color: ${colors.whiteOnBlue5};
      }
      &:active ${StyledFlex} {
        background-color: ${colors.whiteOnBlue10};
      }
    `}

  &&:focus-visible {
    outline: 2px solid ${colors.white};
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
