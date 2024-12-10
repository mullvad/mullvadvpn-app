import { Text, TextProps } from './Text';

export type FoonoteMiniProps = Omit<TextProps, 'variant'>;

export const FootnoteMini = ({ children, ...props }: FoonoteMiniProps) => (
  <Text variant="footnoteMini" {...props}>
    {children}
  </Text>
);
