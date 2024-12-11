import { Text, TextProps } from './Text';
export type TitleBigProps = Omit<TextProps, 'variant'>;

export const TitleBig = ({ children, ...props }: TitleBigProps) => (
  <Text variant="titleBig" {...props}>
    {children}
  </Text>
);
