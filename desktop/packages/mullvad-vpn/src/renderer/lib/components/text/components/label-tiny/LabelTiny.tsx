import { Text, TextProps } from '../../Text';

export type LabelTinyProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function LabelTiny<T extends React.ElementType = 'span'>(props: LabelTinyProps<T>) {
  return <Text variant="labelTiny" {...props} />;
}
