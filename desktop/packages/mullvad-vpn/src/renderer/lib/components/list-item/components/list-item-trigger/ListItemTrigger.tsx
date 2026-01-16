import { forwardRef } from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { useListItemContext } from '../../ListItemContext';
import { StyledListItemItem } from '../list-item-item';

export const StyledListItemTrigger = styled.button`
  display: flex;
  width: 100%;
  background-color: transparent;

  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
    z-index: 10;
  }

  ${({ disabled }) => {
    if (!disabled) {
      return css`
        &:hover {
          ${StyledListItemItem} {
            background-color: ${colors.whiteOnBlue10};
          }
        }

        &:active {
          ${StyledListItemItem} {
            background-color: ${colors.whiteOnBlue20};
          }
        }
      `;
    }

    return null;
  }}
`;

export type ListItemTriggerProps = React.HtmlHTMLAttributes<HTMLButtonElement>;

export const ListItemTrigger = forwardRef<HTMLButtonElement, ListItemTriggerProps>((props, ref) => {
  const { disabled } = useListItemContext();
  return <StyledListItemTrigger ref={ref} disabled={disabled} {...props} />;
});

ListItemTrigger.displayName = 'ListItemTrigger';
