import { KnownTarget } from 'styled-components/dist/types';

import { Text, TextProps } from './Text';
export type LabelTinyProps = Omit<TextProps<KnownTarget>, 'variant'>;

export const LabelTiny = ({ children, ...props }: LabelTinyProps) => (
  <Text variant="labelTiny" {...props}>
    {children}
  </Text>
);
