import styled from 'styled-components';

import { ListItem } from '../../../../../list-item';
import { ListItemIconProps } from '../../../../../list-item/components';
import { useListboxOptionContext } from '../../';

export type ListboxOptionIconProps = ListItemIconProps;

export const StyledListboxOptionIcon = styled(ListItem.Icon)<{ $selected: boolean }>`
  visibility: ${({ $selected }) => ($selected ? 'visible' : 'hidden')};
`;

export function ListboxOptionIcon(props: ListboxOptionIconProps) {
  const { selected } = useListboxOptionContext();

  return (
    <StyledListboxOptionIcon color={selected ? 'green' : 'white'} $selected={selected} {...props} />
  );
}
