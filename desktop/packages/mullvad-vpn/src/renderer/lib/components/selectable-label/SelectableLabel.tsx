import styled, { css } from 'styled-components';

import { Flex } from '../flex';
import { ListItem } from '../list-item';
import { Text, type TextProps } from '../text';
import { useSelectableLabelColor } from './hooks';

export type SelectableLabelProps<E extends React.ElementType = 'span'> = TextProps<E> & {
  selected?: boolean;
  disabled?: boolean;
};

export const StyledFlex = styled(Flex)`
  position: relative;
`;

export const StyledSelectableLabelIcon = styled(ListItem.Icon)<{ $selected: boolean }>`
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

export const SelectableLabel = <E extends React.ElementType = 'span'>({
  selected = false,
  disabled = false,
  ...props
}: SelectableLabelProps<E>) => {
  const color = useSelectableLabelColor(selected, disabled);
  return (
    <StyledFlex>
      <StyledSelectableLabelIcon icon="checkmark" color={color} $selected={selected} />
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
