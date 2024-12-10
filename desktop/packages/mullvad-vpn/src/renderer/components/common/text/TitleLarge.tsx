import { Text, TextProps } from './Text';
export type TitleLargeProps = Omit<TextProps, 'variant'>;

export const TitleLarge = ({ children, ...props }: TitleLargeProps) => (
  <Text variant="titleLarge" {...props}>
    {children}
  </Text>
);
