import React from 'react';
import styled from 'styled-components';

import { DeprecatedColors, Typography, typography } from '../../foundations';
import { PolymorphicProps, TransientProps } from '../../types';

type TextBaseProps = {
  variant?: Typography;
  color?: DeprecatedColors;
};

export type TextProps<T extends React.ElementType = 'span'> = PolymorphicProps<T, TextBaseProps>;

const StyledText = styled.span<TransientProps<TextBaseProps>>(
  ({ $variant = 'bodySmall', $color = DeprecatedColors.white }) => {
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

export const Text = <T extends React.ElementType = 'span'>({
  variant,
  color,
  ...props
}: TextProps<T>) => {
  return <StyledText $variant={variant} $color={color} {...props} />;
};

Text.displayName = 'Text';
