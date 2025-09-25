import { Text, TextProps } from '../../Text';

export type TitleBigProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function TitleBig<T extends React.ElementType = 'span'>(props: TitleBigProps<T>) {
  return <Text variant="titleBig" {...props} />;
}
