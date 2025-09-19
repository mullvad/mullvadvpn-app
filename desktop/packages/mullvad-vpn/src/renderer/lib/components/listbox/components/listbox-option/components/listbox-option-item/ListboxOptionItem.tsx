import styled, { css } from 'styled-components';

import { colors } from '../../../../../../foundations';
import { ListItem } from '../../../../../list-item';
import { ListItemItemProps } from '../../../../../list-item/components';
import { useListboxContext } from '../../../../';
import { useListboxOptionContext } from '../../';

export type ListItemOptionItemProps = ListItemItemProps;

export const StyledListItemOptionItem = styled(ListItem.Item)<{ $selected: boolean }>`
  ${({ $selected }) => {
    return css`
      background-color: ${$selected ? colors.green : undefined};
    `;
  }}
`;

export function ListboxOptionItem(props: ListItemOptionItemProps) {
  const { value: selectedValue } = useListboxContext();
  const { value } = useListboxOptionContext();
  const selected = value === selectedValue;
  return <StyledListItemOptionItem $selected={selected} {...props} />;
}
