import { createElement, forwardRef } from 'react';
import styled, { WebTarget } from 'styled-components';

import { Colors, Typography, typography, TypographyProperties } from '../../foundations';
import { TransientProps } from '../../types';

export type TextProps = React.PropsWithChildren<{
  variant?: Typography;
  color?: Colors;
  tag?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'p' | 'span';
  as?: WebTarget;
  style?: React.CSSProperties;
}>;

const StyledText = styled(
  ({ tag = 'span', ...props }: { tag: TextProps['tag'] } & TransientProps<TypographyProperties>) =>
    createElement(tag, props),
)((props) => ({
  color: 'var(--color)',
  fontFamily: props.$fontFamily,
  fontWeight: props.$fontWeight,
  fontSize: props.$fontSize,
  lineHeight: props.$lineHeight,
}));

export const Text = forwardRef(
  (
    {
      tag = 'span',
      variant = 'bodySmall',
      color = Colors.white,
      children,
      style,
      ...props
    }: TextProps,
    ref,
  ) => {
    const { fontFamily, fontSize, fontWeight, lineHeight } = typography[variant];
    return (
      <StyledText
        ref={ref}
        tag={tag}
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
