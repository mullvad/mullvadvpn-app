import styled, { css } from 'styled-components';

import {
  ListItem,
  ListItemPositions,
  type ListItemProps,
  StyledListItemRoot,
} from '../../../../../list-item';
import { ListboxOptionItem, ListboxOptionTrigger, StyledListboxOptionItem } from './components';
import { ListboxOptionProvider } from './ListboxOptionContext';

export type ListboxOptionProps<T> = ListItemProps & {
  value: T;
};

export const StyledListboxOption = styled(ListItem)<{
  $position?: ListItemPositions;
}>`
  ${({ $position }) => css`
    --disabled-border-radius: 0;

    ${() => {
      if ($position === 'auto') {
        return css`
          // If it is the first child is followed by another option
          &:first-child:has(+ ${StyledListItemRoot}) {
            ${StyledListboxOptionItem} {
              border-top-left-radius: var(--disabled-border-radius);
              border-top-right-radius: var(--disabled-border-radius);
            }
          }
        `;
      }

      return null;
    }}
  `}
`;

function ListboxOption<T>({ value, position = 'auto', children, ...props }: ListboxOptionProps<T>) {
  return (
    <ListboxOptionProvider value={value}>
      <StyledListboxOption level={1} $position={position} position={position} {...props}>
        {children}
      </StyledListboxOption>
    </ListboxOptionProvider>
  );
}

const ListboxOptionNamespace = Object.assign(ListboxOption, {
  Trigger: ListboxOptionTrigger,
  Item: ListboxOptionItem,
});

export { ListboxOptionNamespace as ListboxOption };
