import { WebTarget } from 'styled-components';

import { Text, TextProps } from './Text';

export type FoonoteMiniProps = Omit<TextProps<WebTarget>, 'variant'>;

export const FootnoteMini = ({ children, ...props }: FoonoteMiniProps) => (
  <Text variant="footnoteMini" {...props}>
    {children}
  </Text>
);
