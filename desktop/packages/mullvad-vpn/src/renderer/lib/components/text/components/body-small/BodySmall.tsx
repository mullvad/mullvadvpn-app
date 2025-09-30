import { Text, TextProps } from '../../Text';

export type BodySmallProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function BodySmall<T extends React.ElementType = 'span'>(props: BodySmallProps<T>) {
  return <Text variant="bodySmall" {...props} />;
}
