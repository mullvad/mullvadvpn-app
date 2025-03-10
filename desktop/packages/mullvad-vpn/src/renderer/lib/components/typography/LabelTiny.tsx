import { WebTarget } from 'styled-components';

import { Text, TextProps } from './Text';
export type LabelTinyProps = Omit<TextProps<WebTarget>, 'variant'>;

export const LabelTiny = ({ children, ...props }: LabelTinyProps) => (
  <Text variant="labelTiny" {...props}>
    {children}
  </Text>
);
