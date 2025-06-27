import React from 'react';
import styled from 'styled-components';

import { Colors, colors, Typography, typography } from '../../foundations';
import { PolymorphicProps, TransientProps } from '../../types';

type TextBaseProps = {
  variant?: Typography;
  color?: Colors;
  textAlign?: React.CSSProperties['textAlign'];
};

export type TextProps<T extends React.ElementType = 'span'> = PolymorphicProps<T, TextBaseProps>;

const StyledText = styled.span<TransientProps<TextBaseProps>>(
  ({ $variant = 'bodySmall', $color = 'white', $textAlign }) => {
    const { fontFamily, fontSize, fontWeight, lineHeight } = typography[$variant];
    const color = colors[$color];
    return `
      --color: ${color};

      color: var(--color);
      text-align: ${$textAlign || undefined};
      font-family: ${fontFamily};
      font-size: ${fontSize};
      font-weight: ${fontWeight};
      line-height: ${lineHeight};
    `;
  },
);

export const Text = <T extends React.ElementType = 'span'>({
  variant,
  color,
  textAlign,
  ...props
}: TextProps<T>) => {
  return <StyledText $variant={variant} $color={color} $textAlign={textAlign} {...props} />;
};

Text.displayName = 'Text';
