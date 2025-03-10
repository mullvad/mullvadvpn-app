import { KnownTarget } from 'styled-components/dist/types';

import { Text, TextProps } from './Text';

export type BodySmallProps = Omit<TextProps<KnownTarget>, 'variant'>;

export const BodySmall = ({ children, ...props }: BodySmallProps) => (
  <Text variant="bodySmall" {...props}>
    {children}
  </Text>
);
