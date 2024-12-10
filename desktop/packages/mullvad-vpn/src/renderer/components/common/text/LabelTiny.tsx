import { Text, TextProps } from './Text';
export type LabelTinyProps = Omit<TextProps, 'variant'>;

export const LabelTiny = ({ children, ...props }: LabelTinyProps) => (
  <Text variant="labelTiny" {...props}>
    {children}
  </Text>
);
