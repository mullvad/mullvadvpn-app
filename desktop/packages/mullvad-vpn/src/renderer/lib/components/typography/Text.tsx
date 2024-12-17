import { forwardRef } from 'react';
import styled from 'styled-components';

import { Colors, Typography, typography, TypographyProperties } from '../../foundations';
import { TransientProps } from '../../styles';

export type TextProps = React.PropsWithChildren<{
  variant?: Typography;
  color?: Colors;
  as?: React.ElementType;
  style?: React.CSSProperties;
}>;

const StyledText = styled.span<TransientProps<TypographyProperties>>((props) => ({
  color: 'var(--color)',
  fontFamily: props.$fontFamily,
  fontWeight: props.$fontWeight,
  fontSize: props.$fontSize,
  lineHeight: props.$lineHeight,
}));

export const Text = forwardRef(
  ({ variant = 'bodySmall', color = Colors.white, children, style, ...props }: TextProps, ref) => {
    const { fontFamily, fontSize, fontWeight, lineHeight } = typography[variant];
    return (
      <StyledText
        ref={ref}
        style={
          {
            '--color': color,
            ...style,
          } as React.CSSProperties
        }
        $fontFamily={fontFamily}
        $fontWeight={fontWeight}
        $fontSize={fontSize}
        $lineHeight={lineHeight}
        {...props}>
        {children}
      </StyledText>
    );
  },
);

Text.displayName = 'Text';
