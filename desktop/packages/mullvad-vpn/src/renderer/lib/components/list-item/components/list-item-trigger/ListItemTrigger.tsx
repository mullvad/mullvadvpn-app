import styled, { css } from 'styled-components';

import { Colors } from '../../../../foundations';
import { ButtonBase } from '../../../button';
import { ListItemProps } from '../../ListItem';
import { useListItem } from '../../ListItemContext';

// TODO: Colors should be replace with
// with new color tokens once they are implemented.
const StyledButton = styled(ButtonBase)<Pick<ListItemProps, 'disabled'>>`
  display: flex;
  width: 100%;
  --background: transparent;
  background-color: var(--background);

  &&:focus-visible {
    outline: 2px solid ${Colors.white};
    outline-offset: -2px;
    z-index: 10;
  }

  ${({ disabled }) => {
    if (!disabled) {
      return css`
        --background: rgba(41, 77, 115, 1);

        &:hover {
          --background: rgba(62, 95, 129, 1);
          background-color: var(--background);
        }

        &:active {
          --background: rgba(84, 113, 143, 1);
          background-color: var(--background);
        }
      `;
    }

    return null;
  }}
`;

export type ListItemTriggerProps = React.HtmlHTMLAttributes<HTMLButtonElement>;

export const ListItemTrigger = (props: ListItemTriggerProps) => {
  const { disabled } = useListItem();
  return <StyledButton disabled={disabled} {...props} />;
};
