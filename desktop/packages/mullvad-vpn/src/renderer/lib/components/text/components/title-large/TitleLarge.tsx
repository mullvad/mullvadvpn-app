import { Text, TextProps } from '../../Text';

export type TitleLargeProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function TitleLarge<T extends React.ElementType = 'span'>(props: TitleLargeProps<T>) {
  return <Text variant="titleLarge" {...props} />;
}
