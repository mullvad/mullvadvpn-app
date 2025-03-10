import { Text, TextProps } from './Text';

export type BodySmallSemiBoldProps<E extends React.ElementType> = TextProps<E>;

export const BodySmallSemiBold = <T extends React.ElementType = 'span'>(
  props: BodySmallSemiBoldProps<T>,
) => <Text variant="bodySmallSemibold" {...props} />;
