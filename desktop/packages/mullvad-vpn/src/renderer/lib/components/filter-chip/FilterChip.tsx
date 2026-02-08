import styled, { css } from 'styled-components';

import { colors, Radius } from '../../foundations';
import type { PolymorphicProps } from '../../types';
import { Flex } from '../flex';
import { FilterChipIcon, FilterChipText } from './components';
import { FilterChipProvider } from './FilterChipContext';

export type FilterChipProps<T extends React.ElementType = 'button'> = PolymorphicProps<T>;

const StyledFilterChipButton = styled.button<{ $clickable: boolean }>`
  ${({ $clickable }) => {
    return css`
      --background: ${colors.blue};
      --hover: ${colors.blue60};
      --disabled: ${colors.blue50};

      display: flex;
      align-items: center;

      border-radius: ${Radius.radius8};
      min-width: 32px;
      min-height: 24px;
      background: var(--background);

      ${() => {
        if ($clickable) {
          return css`
            &:not(:disabled):hover {
              background: var(--hover);
            }
          `;
        }
        return null;
      }}

      &:disabled {
        background: var(--disabled);
      }
      &:focus-visible {
        outline: 2px solid ${colors.white};
      }
    `;
  }}
`;

const FilterChip = <T extends React.ElementType = 'button'>({
  children,
  disabled,
  onClick,
  ref,
  ...props
}: FilterChipProps<T>) => {
  return (
    <FilterChipProvider disabled={disabled}>
      <StyledFilterChipButton
        ref={ref}
        disabled={disabled}
        onClick={onClick}
        $clickable={!!onClick}
        {...props}>
        <Flex
          gap="tiny"
          alignItems="center"
          justifyContent="space-between"
          padding={{
            horizontal: 'small',
          }}>
          {children}
        </Flex>
      </StyledFilterChipButton>
    </FilterChipProvider>
  );
};

const FilterChipNamespace = Object.assign(FilterChip, {
  Text: FilterChipText,
  Icon: FilterChipIcon,
});
export { FilterChipNamespace as FilterChip };
