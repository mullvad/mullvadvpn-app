import styled from 'styled-components';

import { ListItem } from '../../../list-item';
import { ListItemItemProps } from '../../../list-item/components';

export type ListItemOptionItemProps = ListItemItemProps;

const StyledListboxItem = styled(ListItem.Item)`
  margin-bottom: 1px;
`;

export function ListboxItem(props: ListItemOptionItemProps) {
  return <StyledListboxItem {...props} />;
}
