import styled from 'styled-components';

import { ListItem } from '../../../list-item';
import { ListItemItemProps } from '../../../list-item/components';

export type ListboxHeaderItemProps = ListItemItemProps;

const StyledListboxItem = styled(ListItem.Item)`
  margin-bottom: 1px;
`;

export function ListboxHeaderItem(props: ListboxHeaderItemProps) {
  return <StyledListboxItem {...props} />;
}
