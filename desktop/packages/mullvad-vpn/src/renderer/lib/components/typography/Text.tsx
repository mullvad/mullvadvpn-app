import React from 'react';
import styled from 'styled-components';

import { Colors, Typography, typography } from '../../foundations';
import { PolymorphicProps, TransientProps } from '../../types';

type TextBaseProps = {
  variant?: Typography;
  color?: Colors;
};

export type TextProps<T extends React.ElementType> = PolymorphicProps<T, TextBaseProps>;

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

export const Text = <T extends React.ElementType = 'span'>({
  variant,
  color,
  ...props
}: TextProps<T>) => {
  return <StyledText $variant={variant} $color={color} {...props} />;
};

Text.displayName = 'Text';
