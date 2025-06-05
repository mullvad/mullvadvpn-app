import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { ButtonBase } from '../../../button';
import { ListItemProps } from '../../ListItem';
import { useListItem } from '../../ListItemContext';

const StyledButton = styled(ButtonBase)<Pick<ListItemProps, 'disabled'>>`
  display: flex;
  width: 100%;
  --background: transparent;
  background-color: var(--background);

  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
    z-index: 10;
  }

  ${({ disabled }) => {
    if (!disabled) {
      return css`
        --background: ${colors.blue};

        &:hover {
          --background: ${colors.whiteOnBlue10};
          background-color: var(--background);
        }

        &:active {
          --background: ${colors.whiteOnBlue20};
          background-color: var(--background);
        }
      `;
    }

    return null;
  }}
`;

export type ListItemTriggerProps = React.HtmlHTMLAttributes<HTMLButtonElement>;

export function ListItemTrigger(props: ListItemTriggerProps) {
  const { disabled } = useListItem();
  return <StyledButton disabled={disabled} {...props} />;
}
