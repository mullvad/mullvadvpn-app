import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../foundations';
import { Trigger, type TriggerProps } from '../trigger';
import { FilterChipIcon, FilterChipText, StyledFilterChipIcon } from './components';
import { FilterChipProvider } from './FilterChipContext';

export type FilterChipProps<T extends React.ElementType = 'button'> = TriggerProps<T> & {
  inactive?: boolean;
};

const variables = {
  background: colors.blue,
  hover: colors.blue60,
  active: colors.blue40,
  inactive: colors.blue20,
  disabled: colors.blue20,
} as const;

export const StyledFilterChip = styled(Trigger)<{ $hasOnClick?: boolean; $inactive?: boolean }>`
  ${({ $hasOnClick, $inactive }) => {
    return css`
      --background: ${variables.background};
      --hover: ${variables.hover};
      --active: ${variables.active};
      --inactive: ${variables.inactive};
      --disabled: ${variables.disabled};

      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: ${spacings.tiny};

      min-width: 32px;
      min-height: 24px;
      padding: ${spacings.tiny} ${spacings.small};
      border-radius: ${Radius.radius8};
      background: var(--background);
      > ${StyledFilterChipIcon} {
        border: 1px solid red;
      }

      ${() => {
        if ($hasOnClick) {
          return css`
            &:not(:disabled) {
              &:hover {
                background-color: var(--hover);
                > ${StyledFilterChipIcon} {
                  background-color: ${colors.whiteAlpha80};
                }
              }
              &:active {
                background-color: var(--active);
                > ${StyledFilterChipIcon} {
                  background-color: ${colors.white};
                }
              }
            }
          `;
        }
        return null;
      }}

      ${() => {
        if ($inactive) {
          return css`
            background-color: var(--inactive);
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

function FilterChip<T extends React.ElementType = 'button'>({
  as,
  children,
  disabled,
  inactive,
  onClick,
  ...props
}: FilterChipProps<T>) {
  return (
    <FilterChipProvider inactive={inactive} disabled={disabled}>
      <StyledFilterChip
        forwardedAs={as}
        disabled={disabled}
        onClick={onClick}
        $inactive={inactive}
        $hasOnClick={onClick !== undefined}
        {...props}>
        {children}
      </StyledFilterChip>
    </FilterChipProvider>
  );
}

const FilterChipNamespace = Object.assign(FilterChip, {
  Text: FilterChipText,
  Icon: FilterChipIcon,
});

export { FilterChipNamespace as FilterChip };
