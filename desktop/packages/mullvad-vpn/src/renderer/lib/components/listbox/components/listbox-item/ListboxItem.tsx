import styled from 'styled-components';

import { ListItem, StyledListItem } from '../../../list-item';
import { ListItemItemProps, StyledListItemItem } from '../../../list-item/components';
import { StyledListboxOptions } from '../listbox-options';

export type ListItemOptionItemProps = ListItemItemProps;

const StyledListboxItem = styled(ListItem.Item)`
  margin-bottom: 1px;
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;

  // If has options as sibling, remove border radius of first option
  && ~ ${StyledListboxOptions} > ${StyledListItem}:first-child ${StyledListItemItem} {
    border-top-left-radius: 0px;
    border-top-right-radius: 0px;
  }
`;

export function ListboxItem(props: ListItemOptionItemProps) {
  return <StyledListboxItem {...props} />;
}
