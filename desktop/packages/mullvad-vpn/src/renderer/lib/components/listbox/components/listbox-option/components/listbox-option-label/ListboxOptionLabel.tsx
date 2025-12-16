import styled, { css } from 'styled-components';

import { Flex, Text, TextProps } from '../../../../..';
import { ListItem } from '../../../../../list-item';
import { useListboxOptionContext } from '../../ListboxOptionContext';
import { useListboxOptionLabelColor } from './hooks';

export type SelectListItemOptionLabelProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const StyledFlex = styled(Flex)`
  position: relative;
`;

export const StyledListboxOptionIcon = styled(ListItem.Icon)<{ $selected: boolean }>`
  ${({ $selected }) => {
    return css`
      --transition-duration: 0.15s;

      position: absolute;
      transition:
        opacity var(--transition-duration) ease-in,
        transform var(--transition-duration) ease-out;
      transform: ${$selected ? 'translateX(0)' : 'translateX(-16px)'};
      opacity: ${$selected ? 1 : 0};
      visibility: ${$selected ? 'visible' : 'hidden'};
    `;
  }}
`;

const StyledText = styled(Text)<{ $selected: boolean }>`
  ${({ $selected }) => css`
    --transition-duration: 0.15s;

    transition:
      transform var(--transition-duration) ease-out,
      color var(--transition-duration) ease-out;
    transform: translateX(${$selected ? 32 : 0}px);
  `}
`;

export const ListboxOptionLabel = <E extends React.ElementType = 'span'>(
  props: SelectListItemOptionLabelProps<E>,
) => {
  const color = useListboxOptionLabelColor();
  const { selected } = useListboxOptionContext();
  return (
    <StyledFlex>
      <StyledListboxOptionIcon icon="checkmark" color={color} $selected={selected} />
      <StyledText
        key="label"
        variant="bodySmallSemibold"
        color={color}
        $selected={selected}
        {...props}
      />
    </StyledFlex>
  );
};
