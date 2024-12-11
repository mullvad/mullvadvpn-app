import { Text, TextProps } from './Text';

export type BodySmallProps = Omit<TextProps, 'variant'>;

export const BodySmall = ({ children, ...props }: BodySmallProps) => (
  <Text variant="bodySmall" {...props}>
    {children}
  </Text>
);
