import { Text, TextProps } from './Text';
export type TitleMediumProps = Omit<TextProps, 'variant'>;

export const TitleMedium = ({ children, ...props }: TitleMediumProps) => (
  <Text variant="titleMedium" {...props}>
    {children}
  </Text>
);
