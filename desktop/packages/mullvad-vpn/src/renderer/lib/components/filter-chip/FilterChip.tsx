import styled, { css, WebTarget } from 'styled-components';

import { colors, Radius, spacings } from '../../foundations';
import { FilterChipIcon, FilterChipText, StyledFilterChipIcon } from './components';
import { FilterChipProvider } from './FilterChipContext';

export interface FilterChipProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  as?: WebTarget;
}

const variables = {
  background: colors.blue,
  hover: colors.blue60,
  active: colors.blue40,
  disabled: colors.blue50,
} as const;

export const StyledFilterChip = styled.button<{ $hasOnClick?: boolean }>`
  ${({ $hasOnClick }) => {
    return css`
      --background: ${variables.background};
      --hover: ${variables.hover};
      --active: ${variables.active};
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

      &:disabled {
        background: var(--disabled);
      }
      &:focus-visible {
        outline: 2px solid ${colors.white};
      }
    `;
  }}
`;

function FilterChip({ children, disabled, style, onClick, ...props }: FilterChipProps) {
  return (
    <FilterChipProvider disabled={disabled}>
      <StyledFilterChip
        disabled={disabled}
        onClick={onClick}
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
