import { KnownTarget } from 'styled-components/dist/types';

import { Text, TextProps } from './Text';

export type BodySmallSemiBoldProps = Omit<TextProps<KnownTarget>, 'variant'>;

export const BodySmallSemiBold = ({ children, ...props }: BodySmallSemiBoldProps) => (
  <Text variant="bodySmallSemibold" {...props}>
    {children}
  </Text>
);
