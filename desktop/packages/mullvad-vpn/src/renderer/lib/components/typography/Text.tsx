import React from 'react';
import styled, { PolymorphicComponentProps, WebTarget } from 'styled-components';

import { Colors, Typography, typography } from '../../foundations';
import { TransientProps } from '../../types';

type TextBaseProps = React.PropsWithChildren<{
  variant?: Typography;
  color?: Colors;
}>;

export type TextProps<T extends WebTarget> = PolymorphicComponentProps<'web', TextBaseProps, T, T>;

const StyledText = styled.span<TransientProps<TextBaseProps>>(
  ({ $variant = 'bodySmall', $color = Colors.white }) => {
    const { fontFamily, fontSize, fontWeight, lineHeight } = typography[$variant];
    return `
      --color: ${$color};
      
      color: var(--color);
      font-family: ${fontFamily};
      font-size: ${fontSize};
      font-weight: ${fontWeight};
      line-height: ${lineHeight};
    `;
  },
);

export const Text = <T extends WebTarget>({
  variant,
  color,
  children,
  style,
  ...props
}: TextProps<T>) => {
  return (
    <StyledText $variant={variant} $color={color} {...props}>
      {children}
    </StyledText>
  );
};

Text.displayName = 'Text';
