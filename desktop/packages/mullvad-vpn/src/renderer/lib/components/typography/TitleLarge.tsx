import { KnownTarget } from 'styled-components/dist/types';

import { Text, TextProps } from './Text';
export type TitleLargeProps = Omit<TextProps<KnownTarget>, 'variant'>;

export const TitleLarge = ({ children, ...props }: TitleLargeProps) => (
  <Text variant="titleLarge" {...props}>
    {children}
  </Text>
);
