import { Text, TextProps } from './Text';

export type BodySmallSemiBoldProps = Omit<TextProps, 'variant'>;

export const BodySmallSemiBold = ({ children, ...props }: BodySmallSemiBoldProps) => (
  <Text variant="bodySmallSemibold" {...props}>
    {children}
  </Text>
);
