import { KnownTarget } from 'styled-components/dist/types';

import { Text, TextProps } from './Text';

export type FoonoteMiniProps = Omit<TextProps<KnownTarget>, 'variant'>;

export const FootnoteMini = ({ children, ...props }: FoonoteMiniProps) => (
  <Text variant="footnoteMini" {...props}>
    {children}
  </Text>
);
