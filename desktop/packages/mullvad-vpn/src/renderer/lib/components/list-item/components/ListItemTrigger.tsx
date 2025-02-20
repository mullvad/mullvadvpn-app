import styled, { css } from 'styled-components';

import { Colors } from '../../../foundations';
import { buttonResetString } from '../../../styles';
import { useListItem } from '../ListItemContext';
import { StyledFlex } from './ListItemContent';

const StyledButton = styled.button<{ $disabled?: boolean }>`
  ${buttonResetString}
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
