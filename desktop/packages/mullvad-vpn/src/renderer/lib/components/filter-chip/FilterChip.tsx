import { forwardRef } from 'react';
import styled, { WebTarget } from 'styled-components';

import { Colors, Radius, Spacings } from '../../foundations';
import { buttonReset } from '../../styles';
import { Flex } from '../flex';
import { FilterChipIcon, FilterChipText } from './components';
import { FilterChipProvider } from './FilterChipContext';

export interface FilterChipProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  as?: WebTarget;
}

const variables = {
  background: Colors.blue,
  hover: Colors.blue60,
  disabled: Colors.blue50,
} as const;

const StyledButton = styled.button({
  ...buttonReset,

  display: 'flex',
  alignItems: 'center',

  borderRadius: Radius.radius8,
  minWidth: '32px',
  minHeight: '24px',
  background: 'var(--background)',
  '&:not(:disabled):hover': {
    background: 'var(--hover)',
  },
  '&:disabled': {
    background: 'var(--disabled)',
  },
  '&:focus-visible': {
    outline: `2px solid ${Colors.white}`,
  },
});

const FilterChip = forwardRef<HTMLButtonElement, FilterChipProps>(
  ({ children, disabled, style, onClick, ...props }, ref) => {
    return (
      <FilterChipProvider disabled={disabled}>
        <StyledButton
          ref={ref}
          style={
            {
              '--background': variables.background,
              '--hover': onClick ? variables.hover : variables.background,
              '--disabled': variables.disabled,
              ...style,
            } as React.CSSProperties
          }
          disabled={disabled}
          onClick={onClick}
          {...props}>
          <Flex
            $flex={1}
            $gap={Spacings.tiny}
            $alignItems="center"
            $justifyContent="space-between"
            $padding={{
              horizontal: Spacings.small,
            }}>
            {children}
          </Flex>
        </StyledButton>
      </FilterChipProvider>
    );
  },
);

FilterChip.displayName = 'FilterChip';

const FilterChipNamespace = Object.assign(FilterChip, {
  Text: FilterChipText,
  Icon: FilterChipIcon,
});
export { FilterChipNamespace as FilterChip };
