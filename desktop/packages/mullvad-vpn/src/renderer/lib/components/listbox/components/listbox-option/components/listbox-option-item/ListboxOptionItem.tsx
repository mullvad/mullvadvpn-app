import styled from 'styled-components';

import { ListItem } from '../../../../../list-item';
import { ListItemItemProps } from '../../../../../list-item/components';

export type ListboxOptionItemProps = ListItemItemProps;

export const StyledListboxOptionItem = styled(ListItem.Item)``;

export function ListboxOptionItem(props: ListboxOptionItemProps) {
  return <StyledListboxOptionItem {...props} />;
}
