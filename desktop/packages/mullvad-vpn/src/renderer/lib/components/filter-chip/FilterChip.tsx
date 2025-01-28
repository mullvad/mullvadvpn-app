import { forwardRef } from 'react';
import styled, { WebTarget } from 'styled-components';

import { Colors, Radius, Spacings } from '../../foundations';
import { buttonReset } from '../../styles';
import { Flex } from '../flex';
import { LabelTiny } from '../typography';

export interface FilterChipProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  trailing?: React.ReactNode;
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

export const FilterChip = forwardRef<HTMLButtonElement, FilterChipProps>(
  ({ trailing, children, disabled, style, onClick, ...props }, ref) => {
    return (
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
          $gap={Spacings.spacing1}
          $justifyContent="space-between"
          $padding={{
            horizontal: Spacings.spacing3,
          }}
          $alignItems="center">
          <Flex $flex={1} $justifyContent="center" $alignItems="center">
            {typeof children === 'string' ? (
              <LabelTiny color={disabled ? Colors.white40 : Colors.white}>{children}</LabelTiny>
            ) : (
              children
            )}
          </Flex>
          {trailing}
        </Flex>
      </StyledButton>
    );
  },
);

FilterChip.displayName = 'Chip';
