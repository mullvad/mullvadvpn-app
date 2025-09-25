import { Text, TextProps } from '../../Text';

export type TitleMediumProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function TitleMedium<T extends React.ElementType = 'span'>(props: TitleMediumProps<T>) {
  return <Text variant="titleMedium" {...props} />;
}
