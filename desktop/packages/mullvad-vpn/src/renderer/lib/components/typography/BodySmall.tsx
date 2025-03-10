import { WebTarget } from 'styled-components';

import { Text, TextProps } from './Text';

export type BodySmallProps = Omit<TextProps<WebTarget>, 'variant'>;

export const BodySmall = ({ children, ...props }: BodySmallProps) => (
  <Text variant="bodySmall" {...props}>
    {children}
  </Text>
);
