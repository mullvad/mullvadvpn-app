import { forwardRef } from 'react';
import styled from 'styled-components';

import { Colors, Typography, typography } from '../../../tokens';

export type TextProps = React.PropsWithChildren<{
  variant?: Typography;
  color?: Colors;
  as?: React.ElementType;
  style?: React.CSSProperties;
}>;

export const StyledText = styled.span({
  color: 'var(--color)',
  fontFamily: 'var(--fontFamily)',
  fontWeight: 'var(--fontWeight)',
  fontSize: 'var(--fontSize)',
  lineHeight: 'var(--lineHeight)',
});

export const Text = forwardRef(
  (
    {
      variant = 'bodySmall',
      color = Colors.white,
      children,
      style,
      ...props
    }: React.PropsWithChildren<TextProps>,
    ref,
  ) => {
    const { fontFamily, fontSize, fontWeight, lineHeight } = typography[variant];
    return (
      <StyledText
        ref={ref}
        style={
          {
            '--color': color,
            '--fontFamily': fontFamily,
            '--fontWeight': fontWeight,
            '--fontSize': fontSize,
            '--lineHeight': lineHeight,
            ...style,
          } as React.CSSProperties
        }
        {...props}>
        {children}
      </StyledText>
    );
  },
);

Text.displayName = 'Text';
