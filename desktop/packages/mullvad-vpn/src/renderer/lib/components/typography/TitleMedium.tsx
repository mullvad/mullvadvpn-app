import { KnownTarget } from 'styled-components/dist/types';

import { Text, TextProps } from './Text';
export type TitleMediumProps = Omit<TextProps<KnownTarget>, 'variant'>;

export const TitleMedium = ({ children, ...props }: TitleMediumProps) => (
  <Text variant="titleMedium" {...props}>
    {children}
  </Text>
);
