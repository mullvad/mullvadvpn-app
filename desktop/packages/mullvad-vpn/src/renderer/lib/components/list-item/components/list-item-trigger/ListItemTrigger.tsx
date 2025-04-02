import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { TransientProps } from '../../../../types';
import { ButtonBase } from '../../../button';
import { ListItemProps } from '../../ListItem';
import { useListItem } from '../../ListItemContext';

// TODO: Colors should be replace with
// with new color tokens once they are implemented.
const StyledButton = styled(ButtonBase)<TransientProps<Pick<ListItemProps, 'disabled'>>>`
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
    `;
  }}
`;

export type ListItemTriggerProps = React.HtmlHTMLAttributes<HTMLButtonElement>;

export const ListItemTrigger = (props: ListItemTriggerProps) => {
  const { disabled } = useListItem();
  return <StyledButton $disabled={disabled} disabled={disabled} {...props} />;
};
