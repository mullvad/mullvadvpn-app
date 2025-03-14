import { Text, TextProps } from './Text';

export type LabelTinyProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const LabelTiny = <T extends React.ElementType = 'span'>(props: LabelTinyProps<T>) => (
  <Text variant="labelTiny" {...props} />
);
