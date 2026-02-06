import styled, { css } from 'styled-components';

import { ListItem, ListItemPositions, ListItemProps, StyledListItemRoot } from '../../../list-item';
import {
  ListboxOptionItem,
  ListboxOptionLabel,
  ListboxOptionTrigger,
  StyledListboxOptionItem,
} from './components';
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
  Group: ListItem.Group,
  Trigger: ListboxOptionTrigger,
  Item: ListboxOptionItem,
  Footer: ListItem.Footer,
  Label: ListboxOptionLabel,
  Checkbox: ListItem.Checkbox,
});

export { ListboxOptionNamespace as ListboxOption };
