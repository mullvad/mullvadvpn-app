import { Text, TextProps } from './Text';

export type BodySmallProps<E extends React.ElementType> = TextProps<E>;

export const BodySmall = <T extends React.ElementType = 'span'>(props: BodySmallProps<T>) => (
  <Text variant="bodySmall" {...props} />
);
