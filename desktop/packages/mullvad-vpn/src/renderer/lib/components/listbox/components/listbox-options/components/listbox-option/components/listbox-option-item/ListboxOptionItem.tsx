import styled from 'styled-components';

import { ListItem } from '../../../../../../../list-item';
import type { ListItemItemProps } from '../../../../../../../list-item/components';
import { ListboxOptionLabel } from './components';

export type ListboxOptionItemProps = ListItemItemProps;

export const StyledListboxOptionItem = styled(ListItem.Item)``;

function ListboxOptionItem(props: ListboxOptionItemProps) {
  return <StyledListboxOptionItem {...props} />;
}

const ListboxOptionItemNamespace = Object.assign(ListboxOptionItem, {
  Label: ListboxOptionLabel,
  Checkbox: ListItem.Item.Checkbox,
  Group: ListItem.Item.Group,
});

export { ListboxOptionItemNamespace as ListboxOptionItem };
