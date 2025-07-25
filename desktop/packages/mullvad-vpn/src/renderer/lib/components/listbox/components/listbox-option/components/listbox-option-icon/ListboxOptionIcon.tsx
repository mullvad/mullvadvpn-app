import styled from 'styled-components';

import { ListItem } from '../../../../../list-item';
import { ListItemIconProps } from '../../../../../list-item/components';
import { useListboxContext } from '../../../listbox-context';
import { useListboxOptionContext } from '../listbox-option-context/ListboxOptionContext';

export type ListboxOptionIconProps = ListItemIconProps;

export const StyledListboxOptionIcon = styled(ListItem.Icon)<{ $selected: boolean }>`
  visibility: ${({ $selected }) => ($selected ? 'visible' : 'hidden')};
`;

export function ListboxOptionIcon(props: ListboxOptionIconProps) {
  const { value: selectedValue } = useListboxContext();
  const { value } = useListboxOptionContext();
  const selected = value === selectedValue;

  return <StyledListboxOptionIcon $selected={selected} {...props} />;
}
