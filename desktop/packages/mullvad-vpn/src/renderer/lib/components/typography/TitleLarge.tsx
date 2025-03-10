import { Text, TextProps } from './Text';
export type TitleLargeProps<E extends React.ElementType> = TextProps<E>;

export const TitleLarge = <T extends React.ElementType = 'span'>(props: TitleLargeProps<T>) => (
  <Text variant="titleLarge" {...props} />
);
