import { Text, TextProps } from '../../Text';

export type BodySmallSemiBoldProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function BodySmallSemiBold<T extends React.ElementType = 'span'>(
  props: BodySmallSemiBoldProps<T>,
) {
  return <Text variant="bodySmallSemibold" {...props} />;
}
