import { WebTarget } from 'styled-components';

import { Text, TextProps } from './Text';
export type TitleLargeProps = Omit<TextProps<WebTarget>, 'variant'>;

export const TitleLarge = ({ children, ...props }: TitleLargeProps) => (
  <Text variant="titleLarge" {...props}>
    {children}
  </Text>
);
