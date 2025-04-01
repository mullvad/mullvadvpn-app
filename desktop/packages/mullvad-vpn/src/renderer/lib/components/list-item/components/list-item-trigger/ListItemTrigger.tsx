import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { ButtonBase } from '../../../button';
import { useListItem } from '../../ListItemContext';
import { StyledFlex } from '../list-item-content';

const StyledButton = styled(ButtonBase)<{ $disabled?: boolean }>`
  display: flex;
  width: 100%;
  ${({ $disabled }) => {
    return css`
      --background: transparent;
      background-color: var(--background);
      ${!$disabled &&
      css`
        --background: ${colors.blue};
        &:hover {
          --background: ${colors.whiteOnBlue10};
          background-color: var(--background);
        }
        &:active {
          --background: ${colors.whiteOnBlue20};
          background-color: var(--background);
        }
      `}
      &&:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: -2px;
        z-index: 10;
      }
      &:active ${StyledFlex} {
        background-color: ${colors.whiteOnBlue10};
      }
    `;
  }}
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
