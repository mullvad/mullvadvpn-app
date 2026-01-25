import React from 'react';
import styled, { css } from 'styled-components';

import { Colors, colors, Typography, typography } from '../../foundations';
import { PolymorphicProps, TransientProps } from '../../types';

type TextBaseProps = {
  variant?: Typography;
  color?: Colors;
  textAlign?: React.CSSProperties['textAlign'];
};

export type TextProps<T extends React.ElementType = 'span'> = PolymorphicProps<T, TextBaseProps>;

export const StyledText = styled.span<TransientProps<TextBaseProps>>(
  ({ $variant = 'bodySmall', $color = 'white', $textAlign }) => {
    const { fontFamily, fontSize, fontWeight, lineHeight } = typography[$variant];
    const color = colors[$color];
    return css`
      --color: ${color};

      color: var(--color);
      font-family: ${fontFamily};
      font-size: ${fontSize};
      font-weight: ${fontWeight};
      line-height: ${lineHeight};
      ${() => {
        if ($textAlign) {
          return css`
            text-align: ${$textAlign};
          `;
        }
        return null;
      }}
    `;
  },
);

export function Text<T extends React.ElementType = 'span'>({
  variant,
  color,
  textAlign,
  ...props
}: TextProps<T>) {
  return <StyledText $variant={variant} $color={color} $textAlign={textAlign} {...props} />;
}
